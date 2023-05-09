//! 可执行文件的缓存系统

#[cfg(all(unix))]

use std::{
	fs,
	path::PathBuf,
	collections::{HashMap, BTreeMap},
};
use core::cmp::Ordering;
use crypto::{sha2::Sha256, digest::Digest};
use crate::{lang::LangOption, error::Error::{self, CacheCE}};
use sandbox::{ExecSandBox, Status as Stat};

/// 缓存系统
pub struct Cache {
	size: u64,         // 最多缓存的文件个数，必须为正整数
	dir: PathBuf,      // 缓存文件夹
	cur_height: u64,   // 当前最大优先级
	map: HashMap<String, Entry>,       // 从哈希值到条目的映射
	sorted: BTreeMap<Entry, String>,   // 从条目到哈希值的有序映射，优先级从小到大
}

/// 缓存条目
#[derive(Clone, Debug)]
struct Entry {
	height: u64,   // 优先级，优先删除值较小的条目
	stat: Stat,    // 编译状态
}

impl Entry {
	fn new(height: u64, stat: Stat) -> Self {
		Self { height, stat }
	}
}

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

fn hash_it(s: String) -> String {
	let mut hasher = Sha256::new();
	hasher.input_str(&s);
	hasher.result_str()
}

impl Cache {
	pub fn new(size: u64, dir: PathBuf) -> Self {
		assert!(size > 0);   // 不太会 Error
		Self {
			size,
			dir,
			cur_height: 0u64,
			map: HashMap::<String, Entry>::new(),
			sorted: BTreeMap::<Entry, String>::new(),
		}
	}
	pub fn get_exec(&mut self, lang: &impl LangOption, src_path: &PathBuf) -> Result<PathBuf, Error> {
		let src = fs::read_to_string(src_path).unwrap();   // 不太会 Error
		let hash = hash_it(hash_it(src) + " " + &hash_it(lang.pure_args().join(" ")));

		let mut dest = self.dir.clone();
		dest.push(&hash);
		
		self.cur_height = self.cur_height + 1;

/*
		eprintln!("");

		for (x, y) in &self.sorted {
			eprintln!("{:?}: \"{}\"", x, y);
		}
*/

		if let Some(entry) = self.map.get_mut(&hash) {
			self.sorted.remove(&entry);
			entry.height = self.cur_height;
			self.sorted.insert(entry.clone(), hash);

			return match &entry.stat {
				Stat::Ok => Ok(dest),
				x => Err(CacheCE(x.clone())),
			};
		}

		if self.map.len() as u64 >= self.size {
			let (_, s) = self.sorted.pop_first().unwrap();
			self.map.remove(&s);
		}

		let cpl = lang.build_sigton(&src_path, &dest);		
		let term = cpl.exec_sandbox().unwrap();   // 不太会 Error
		let st = term.status.clone();
		let entry = Entry::new(self.cur_height, st);
		
		self.map.insert(hash.clone(), entry.clone());
		self.sorted.insert(entry.clone(), hash);
		
		return match entry.stat {
			Stat::Ok => Ok(dest),
			x => Err(CacheCE(x)),
		};
	}
}