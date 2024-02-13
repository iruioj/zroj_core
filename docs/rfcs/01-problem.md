# 有关题目模块（problem）的提案

本提案描述了 problem 模块的功能和推荐实现方案。

## 题目的存储

要求：

- 支持不同方式的题目数据导入、题目数据修改和保存。可以在 `std::fs` 的基础上使用文件系统实现存储（即，开一个文件夹，里面存这个题目的所有文件）。
- 对于题目数据的修改，你需要给出上传文件的格式。例如上传的题目数据压缩包里的命名方式，配置文件的语法（JSON/YAML）和格式等等。
- 需要暴露一些数据的访问接口（只读），分别供前端、评测读取。
- （重要程度较低）需要支持题目的迁移（建议开一个子模块 `problem::migrate`）：UOJ/LOJ/Lemon/Codeforces 等等

建议：

- 考虑到多线程，注意使用 `std::sync::RwLock` 处理好互斥访问。另外同一个文件夹的题目在同一时间应当只存在一个实例去操作它，因此可以设计一个 problemset 对象，用于题目的管理（添加、删除、初始化）。problemset 在全局只能创建一个。请注意，不要暴露一个静态变量供其他模块使用，因为 problemset 的创建不一定是在运行初始化阶段，可能会有一些其他准备工作。**全局只能创建一个**需要由调用者自己保证。

可能的外部调用方式：

```rust
// 可以在 problems 目录下设置一个 manifest 文件，存储题目列表
let probset = ProblemSet::load_dir("/srv/zroj/problems/");
// 获取所有题目
probset.all(); // -> Result<Vec<&Problem>, error>
// 按照 id 获取题目
probset.get(1001); // -> Result<&Problem, error>
// 修改题目（注意互斥锁）
probset.get_mut(1001); // -> Result<&mut Problem, error>
// 新建一个题目，并返回它的可变引用来进行后续初始化
probset.new(); // -> Result<&mut Problem, error>
// 删除这个题目，并返回它本身（onwership 从 probset 拿出来了）
probset.delete(1001); // -> Result<Problem, error>

// Problem::new 可以设为私有方法，仅供 ProblemSet 使用，外部想要调用必须借助 ProblemSet::new()
let p = Problem::new(); // Result<Problem, error>
let p2 = Problem::import(UOJ, path); // 导入其他 OJ 的题目。实现时调用 problem::migrate 里的函数即可
// 获取题目配置
p.config(); // -> Result<&ProblemConfig, error>
let cfg = p.config_mul(); // -> Result<&mut ProblemConfig, error>
cfg.time_limit = 1000;
cfg.memory_limit = 256 * 1024 * 1024;
// 读取题面
p.statement(); // -> Result<&ProblemStatement, error>
// 获取题面的可变引用
let mut stmt = p.statement_mul(); // -> Result<&mut ProblemStatement, error>
// 获取中文题面描述
stmt.describe("zh");
// 获取可变的中文题面描述
stmt.describe_mul("zh", "你好");
p.data();
p.data_mul();
//....
// 所有的 _mul 可以共用一个锁
```

