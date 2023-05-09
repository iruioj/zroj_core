use std::{path::{PathBuf, Path}, fs::{self, File}, io::{self, Read}, collections::HashSet};

use sha2::{Sha256, Digest};
use zip::{ZipArchive, read::ZipFile};

use crate::{ProblemConfig, Builtin, config::Checkable, builtin::{Pair, Single}};

pub struct ProblemSet {
    path: PathBuf, 
    indices: HashSet<u32>
}

impl ProblemSet {
    fn load_dir(path: PathBuf) -> ProblemSet { 
        if !path.exists() || path.is_file() {
            panic!("path not exist or not a folder");
        }
        let mut indices: HashSet<u32> = HashSet::new();
        for target in fs::read_dir(&path).unwrap() {
            let name = String::from(target.unwrap().file_name().as_os_str().to_str().unwrap()); 
            match name.parse::<u32>() {
                Ok(id) => { indices.insert(id); }
                Err(_) => { }
            }
        }
        ProblemSet{ path, indices }
    }

    pub fn validate_path(config: &mut ZipArchive<&File>, target: &PathBuf) -> bool {
        match &config.by_name(target.as_os_str().to_str().unwrap()) {
            Ok(_) => true, 
            Err(_) => false
        }
    }

    fn check<T:Checkable>(zip: &mut ZipArchive<&File>, config: &ProblemConfig<T>) -> Result<(), String> {
        if !ProblemSet::validate_path(zip, &config.checker) || 
            ! match &config.hacker {
                Some(path) => ProblemSet::validate_path(zip, path), 
                None => true
            } || 
            ! match &config.validator {
                Some(path) => ProblemSet::validate_path(zip, path), 
                None => true 
            } {
                return Err(String::from("program path not exist"));
            }
        match &config.tasks {
            crate::Tasks::Subtasks(subtasks, dependencies) => {
                for subtask in subtasks.iter() {
                    for test in subtask.tests.iter() {
                        if !test.check(zip) {
                            return Err(String::from("data path not valid"));
                        }
                    }
                }
                for (predecessor, successor) in dependencies.iter() {
                    if *predecessor > dependencies.len() || 
                        *successor > dependencies.len() {
                        return Err(String::from("dependencies index out of range"))
                    }
                    if *predecessor >= *successor {
                        return Err(String::from("predecessor not less than successor"))
                    }
                }
            }
            crate::Tasks::TestCases(testcases) => {
                for testcase in testcases.iter() {
                    let test = &testcase.test; 
                    if !test.check(zip) {
                        return Err(String::from("data path not valid"));
                    }
                }
            }
        }
        Ok(())
    }

    fn read_config(config: &mut ZipFile) -> Result<Builtin, String> {
        let mut result = String::from("");
        match config.read_to_string(&mut result) {
            Ok(_) => (), 
            Err(_) => return Result::Err(String::from("error when open config.json, maybe it does not exist"))
        };
        Result::Ok(serde_json::from_str(&result).unwrap())
    } 

    fn inspect(zip: &mut ZipArchive<&File>) -> Result<(), String> {
        match zip.clone().by_name("config.json") {
            Ok(mut zipfile) => {
                let data = match ProblemSet::read_config(&mut zipfile) {
                    Ok(ok) => ok, 
                    Err(err) => return Err(err)
                };
                match data {
                    Builtin::Traditional(config) => ProblemSet::check::<Pair>(zip, &config).clone(), 
                    Builtin::Interactive(config) => ProblemSet::check::<Pair>(zip, &config).clone(), 
                    Builtin::AnswerOnly(config) => ProblemSet::check::<Single>(zip, &config).clone()
                }
            }
            Err(_) => { return Result::Err(String::from("error when open config.json, maybe it does not exist")); }
        }
    }

    fn extract(from: &PathBuf, to: &PathBuf) -> Result<String, String> {
        if !to.exists() {
            println!("Info: target path does not exist, trying to recursively create the directory.");
            fs::create_dir_all(to).unwrap();
        } else {
            println!("Info: target path exists, clearing all assets...");
            fs::remove_dir_all(to).unwrap();
            fs::create_dir(to).unwrap();
        }
        if !from.exists() {
            return Result::Err(String::from("file not exist"));
        }
        let mut zipfile = std::fs::File::open(&from).unwrap();
        let mut zip = zip::ZipArchive::new(&zipfile).unwrap();
        for i in 0..zip.len() {
            let mut file = zip.by_index(i).unwrap();
            if file.is_dir() {
                let target = to.join(Path::new(&file.name().replace("\\", "")));
                fs::create_dir_all(target).unwrap();
            } else {
                let target = to.join(Path::new(&file.name()));
                let mut targetfile = 
                    if !target.exists() {
                        fs::File::create(target).unwrap()
                    } else {
                        fs::File::open(target).unwrap()
                    };
                io::copy(&mut file, &mut targetfile).unwrap();
            }
        }
        let mut hasher = Sha256::new();
        io::copy(&mut zipfile, &mut hasher).unwrap();
        let hash = format!("{:x}", hasher.finalize());
        return Result::Ok(hash);
    }

    fn dir_name(&self, id: u32) -> PathBuf {
        return self.path.clone().join("/").join(id.to_string()).join("/");
    }

    /// 添加题目并上传压缩配置文件，locate 表示文件位置，id 表示题目编号。返回压缩文件的 sha256
    pub fn add(&self, locate: &PathBuf, id: u32) -> Result<String, String> {
        if self.indices.contains(&id) {
            return Result::Err(String::from("index already exist"))
        }
        let zipfile = std::fs::File::open(&locate).expect("zipfile not exist");
        let mut zip = zip::ZipArchive::new(&zipfile).unwrap();
        match ProblemSet::inspect(&mut zip) {
            Ok(_) => ProblemSet::extract(locate, &self.dir_name(id)),
            Err(err) => Err(err)
        }
    }

    /// 更新题目配置文件
    pub fn update(&self, locate: &PathBuf, id: u32) -> Result<String, String> {
        if !self.indices.contains(&id) {
            return Err(String::from("index not exist"));
        }
        let zipfile = std::fs::File::open(&locate).expect("zipfile not exist");
        let mut zip = zip::ZipArchive::new(&zipfile).unwrap();
        match ProblemSet::inspect(&mut zip) {
            Ok(_) => ProblemSet::extract(locate, &self.dir_name(id)),
            Err(err) => Err(err)
        }
    }

    /// 给定题目编号，获取题目信息
    pub fn get_detail(&self, id: u32) -> Result<Builtin, String> {
        if !self.indices.contains(&id) {
            return Err(String::from("index not exist"));
        }
        let mut config = fs::File::open(self.dir_name(id).join("config.json")).unwrap();
        let mut result = String::from("");
        config.read_to_string(&mut result).unwrap();
        Result::Ok(serde_json::from_str(&result).unwrap())
    }
}
