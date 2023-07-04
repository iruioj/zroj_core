//! 可执行文件和编译日志的缓存系统

use std::{ collections::{BTreeMap, HashMap}, io::Read };
use core::cmp::Ordering;
use store::Handle;
use crate::{
    error::Error,
    lang::Compile,
    seq_hash,
	store_file::StoreFile,
};

/// 缓存条目
#[derive(Clone, Debug)]
struct Entry {
    height: u64,           // 优先级；缓存满了，优先删除值最小的条目（所有条目优先级互不相同）
    stat: sandbox::Status, // 编译结果
}
/// 新建缓存条目
impl Entry {
	fn new(height: u64, stat: sandbox::Status) -> Self {
		Self { height, stat }
	}
}
/// 作为 BTreeMap 键值，需要用到的一些比较
impl Ord for Entry {
	fn cmp(&self, other: &Self) -> Ordering {
		self.height.cmp(&other.height)
	}
}
impl PartialOrd for Entry {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}
impl PartialEq for Entry {
	fn eq(&self, other: &Self) -> bool {
		self.height == other.height
	}
}
impl Eq for Entry {}

/// 缓存系统
pub struct Cache {
    size: u64,                       // 最多缓存的文件个数，必须为正整数
    dir: Handle,                     // 缓存文件夹
    cur_height: u64,                 // 当前最大优先级，也是最新一个条目对应的优先级
    map: HashMap<String, Entry>,     // 从哈希值到条目的映射
    sorted: BTreeMap<Entry, String>, // 从条目到哈希值的有序映射，优先级从小到大
}

/// 编译结果
pub struct CompileResult {
	pub stat: sandbox::Status, // 沙盒状态
	pub exec: Option<Handle>,  // 可执行文件
	pub clog: Handle,          // 编译日志
}

impl Cache {
	/// 新建缓存系统，最多缓存的文件个数不能为零
    pub fn new(size: u64, dir: Handle) -> Self {
		assert!(size != 0);
		Self {
			size,
			dir,
			cur_height: 0,
			map: HashMap::<String, Entry>::new(),
			sorted: BTreeMap::<Entry, String>::new(),
		}
    }
	/// 传入源文件，获取可执行文件和编译日志
    pub fn compile(&mut self, src: &mut StoreFile) -> Result<CompileResult, Error> {
        self.cur_height += 1;
		
        let mut src_content = "".into();
		src.file.read_to_string(&mut src_content)?;
		let lang = &src.file_type;
        let hash = seq_hash![src_content, lang];
        let exec = self.dir.clone().join(&hash);
		let clog = exec.clone().join(".c.log");

        if let Some(entry) = self.map.get_mut(&hash) {
            self.sorted.remove(entry);
            entry.height = self.cur_height;
            self.sorted.insert(entry.clone(), hash);
            return Ok(match &entry.stat {
                sandbox::Status::Ok => CompileResult {
					stat: sandbox::Status::Ok,
					exec: Some(exec),
					clog,
				},
                x => CompileResult {
					stat: x.clone(),
					exec: None,
					clog,	
				}
            });
        }

        if self.map.len() as u64 >= self.size {
            let (_, s) = self.sorted.pop_first().unwrap();
            self.map.remove(&s);
        }

		let src_path = self.dir.join("main").join(src.file_type.ext());
		// !!! TODO !!! 处理文件错误
		src.copy_to(&src_path).unwrap();
        let cpl = src.file_type.compile_sandbox(&src_path, &exec, &clog);
		// !!! TODO !!! 处理文件错误
		src_path.remove_file().unwrap();
        let term = cpl.exec_sandbox()?;
        let entry = Entry::new(self.cur_height, term.status);

        self.map.insert(hash.clone(), entry.clone());
        self.sorted.insert(entry.clone(), hash);

		return Ok(match entry.stat {
			sandbox::Status::Ok => CompileResult {
				stat: sandbox::Status::Ok,
				exec: Some(exec),
				clog,
			},
			x => CompileResult {
				stat: x,
				exec: None,
				clog,	
			}
		});
    }
}
