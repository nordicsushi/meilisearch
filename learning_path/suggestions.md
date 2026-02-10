# Meilisearch 学习路径 & 贡献建议

## 🎯 推荐学习路径

### 阶段 1：从小型工具 crate 入手（提升基础 Rust）

从以下独立性强、代码量小的 crate 开始：

| Crate | 学习重点 | 代码量 |
|-------|---------|--------|
| `permissive-json-pointer` | Rust 字符串操作、迭代器 | 小 |
| `flatten-serde-json` | Serde 序列化、递归处理 | 小 |
| `json-depth-checker` | 递归算法、性能优化 | 小 |
| `meili-snap` | 宏（macro）编写、测试框架 | 中 |

**这些 crate 的优点**：逻辑独立，可以完整理解，能快速看到 Rust 的 pattern matching、迭代器、错误处理等惯用法。

### 阶段 2：理解核心类型系统

深入 `meilisearch-types` crate：
- 学习如何设计跨 crate 共享的类型
- 理解 Rust 的 trait 设计模式
- 看 Error 类型的定义（thiserror 用法）

### 阶段 3：攻坚核心引擎

`milli` crate 是精华，但复杂度高。建议：
1. 先读 `milli/src/search/` - 搜索流程
2. 再读 `milli/src/update/` - 索引流程
3. 学习 `heed`（LMDB wrapper）的事务处理

### 阶段 4：HTTP 层 & 异步

`meilisearch` 和 `index-scheduler`：
- Actix-web 框架使用
- 异步编程（async/await）
- 任务队列设计

---

## 🚀 Contribution 建议

1. **立即可做**：去 GitHub 找 `good first issue` 标签
2. **文档贡献**：发现代码注释不清晰的地方，提 PR 改进
3. **测试补充**：项目用 `insta` 做快照测试，给未覆盖的边界情况补测试
4. **小工具 crate**：`permissive-json-pointer`、`flatten-serde-json` 等代码量小，更容易找到改进点

---

## 📝 实践建议

```bash
# 1. 先跑通测试，确保环境 OK
cargo test -p permissive-json-pointer

# 2. 用 MEILI_TEST_FULL_SNAPS 看完整快照
export MEILI_TEST_FULL_SNAPS=true

# 3. 加速编译
export LINDERA_CACHE=$HOME/.cache/meili/lindera
```

### 🧪 理解 Snapshot Tests

Meilisearch 使用 `insta` 库进行快照测试：

**传统 assert vs Snapshot**

```rust
// 传统方式：手动写预期值
assert_eq!(result, "expected output");

// Snapshot 方式：自动保存输出
insta::assert_snapshot!(result);
```

**工作流程：**
1. 第一次运行测试 → 生成 `snapshots/*.snap` 文件
2. 提交 `.snap` 文件到 Git（就像普通代码）
3. 后续测试自动对比输出 vs `.snap` 文件
4. 输出变化时：`cargo insta review` 查看 diff 并批准/拒绝

**CI 中的运行：**
- CI 从 Git 读取已提交的 `.snap` 文件
- 运行测试自动对比，不一致则失败
- 无需额外配置

**优势：**
- ✅ 适合复杂输出（JSON、搜索结果）
- ✅ 代码变更可视化（通过 Git diff）
- ✅ 批量更新（不用逐个改 assert）

### 🛠️ 理解 Build Tools (cargo xtask)

`crates/xtask/` 是项目的自动化工具箱：

```bash
cargo xtask --help           # 查看所有命令
cargo xtask list-features    # 列出所有 feature flags
cargo xtask bench            # 运行性能测试
cargo xtask test             # 自定义测试流程
```

**作用：** 用 Rust 代替 shell 脚本写自动化任务，跨平台且可复用项目代码。

---

## 💡 核心建议（已修正）

### ⚠️ 如果你 Rust 刚起步

**不要直接做 good first issue！原因：**
- Good first issue ≠ Rust 新手友好，只是"对熟悉项目的人容易"
- 你会同时面对：Rust 语法 + 项目架构，双重挑战
- 卡住时无法判断是语言问题还是业务逻辑问题

### ✅ 修正后的单线程学习路径

**阶段 A：只读代码，不想贡献（1-2 周）**

1. **精读一个小 crate**：选 `permissive-json-pointer` 或 `flatten-serde-json`（约 300-500 行）
2. **边读边注释**：在代码里加中文注释，写下每个函数的理解
3. **跑测试学习**：看测试用例，理解 Rust 的测试写法和项目风格
4. **目标**：能用自己的话完整解释这个 crate 的功能

**阶段 B：带着理解看 Issue（之后）**

读完一个小 crate 后，你会：
- ✅ 熟悉项目代码风格
- ✅ 知道测试怎么写（insta snapshot 风格）
- ✅ 理解项目的 error handling 模式

这时再看 good first issue，你能快速判断：
- 这个 issue 涉及哪些 crate？
- 我学过的知识能用上吗？
- 改动范围在我理解范围内吗？

### 🎯 推荐起点

从 `permissive-json-pointer` 开始，因为：
- 代码量小（约 300 行）
- 逻辑清晰（JSON Pointer RFC 标准的宽松实现）
- 无复杂依赖
- 测试覆盖好，能学到项目测试风格

```bash
# 进入目录，读源码
cd crates/permissive-json-pointer
cargo test  # 跑测试理解功能
```

---

## ✍️ 从读代码到写代码的过渡

### 第一步：边读边改（7-10 天）

不要只看代码，要**破坏性修改**：

```rust
// 原代码
fn parse_token(s: &str) -> Option<Token> { ... }

// 实验：改返回类型看看会怎样
fn parse_token(s: &str) -> Result<Token, String> { ... }
```

- 看编译器报错（Rust 编译器是最好的老师）
- 修复所有报错点
- 理解为什么原设计更好
- 改回来

### 第二步：写测试（最重要！⭐）

**别先写新功能，先给已有代码加测试。**

在 `crates/permissive-json-pointer/src/lib.rs` 中找一个没测试覆盖的边界情况：

```rust
#[test]
fn test_empty_pointer() {
    let ptr = JsonPointer::parse("");
    insta::assert_debug_snapshot!(ptr);
}

#[test]
fn test_special_characters() {
    let ptr = JsonPointer::parse("/~0~1");  // ~0 = ~, ~1 = /
    insta::assert_debug_snapshot!(ptr);
}
```

**你会学到：**
- ✅ 如何调用函数
- ✅ 如何构造测试数据
- ✅ 如何用 `insta` 写测试
- ✅ 如何运行 `cargo insta review`

### 第三步：微小改进（文档优先）

找**文档/注释**相关的改进（容易被接受的 PR）：

```rust
// Before
pub fn parse(s: &str) -> Result<Self, Error> {

// After
/// Parses a JSON Pointer string according to RFC 6901 (permissive mode).
///
/// Unlike strict RFC compliance, this parser allows:
/// - Unescaped special characters in tokens
/// - Relaxed validation rules
///
/// # Examples
/// ```
/// use permissive_json_pointer::JsonPointer;
/// let ptr = JsonPointer::parse("/foo/bar")?;
/// ```
pub fn parse(s: &str) -> Result<Self, Error> {
```

**其他改进方向：**
- 函数缺少 doc comment？加上
- README 例子不够清晰？改善
- 错误信息太模糊？提供更多上下文

### 第四步：修复简单 Bug 或添加功能

这时再去看 good first issue，你已经会：
- ✅ 读懂代码
- ✅ 写测试
- ✅ 改代码不破坏现有功能
- ✅ 按项目规范提交 PR

---

## 📅 具体 14 天行动计划

| 时间 | 任务 | 输出 | 成功标准 |
|------|------|------|---------|
| **Day 1-3** | 精读 `permissive-json-pointer` 全部代码 | 代码中文注释版本 | 能用自己话解释每个函数 |
| **Day 4-5** | 跑所有测试，理解每个测试的意图 | 测试笔记 | 知道每个测试在验证什么边界情况 |
| **Day 6-8** | 给 2-3 个函数添加边界测试 | 本地测试全部通过 | `cargo test` 成功 + snapshot review 通过 |
| **Day 9-10** | 改进 1-2 处文档或注释 | 本地 `cargo doc --open` 看起来更好 | 文档描述清晰准确 |
| **Day 11-12** | 准备第一个 PR（测试或文档） | GitHub PR draft | 描述清晰，改动范围小 |
| **Day 13-14** | 根据 review 反馈修改 | PR 合并或明确下一步 | 完成贡献流程体验 |

### 🎯 里程碑检查点

- ✅ **Day 3 检查点**：能不看代码解释 `JsonPointer::parse()` 做了什么？
- ✅ **Day 8 检查点**：写的测试能通过 `cargo test` 和 `cargo insta review`？
- ✅ **Day 10 检查点**：改进的文档让新手更容易理解？
- ✅ **Day 14 检查点**：第一个 PR 提交了吗？

### 💡 关键心态

- **不要追求完美**：第一个 PR 可能只改 5 行文档，这很正常
- **接受 rejection**：PR 被拒绝是学习机会，不是失败
- **小步快跑**：10 个小 PR 比 1 个大 PR 更容易被接受
- **问问题**：在 PR 或 Discord 问问题，maintainer 通常很友好

---

**开始时间：今天！**  
**第一个任务：** `cd crates/permissive-json-pointer && cargo test`
