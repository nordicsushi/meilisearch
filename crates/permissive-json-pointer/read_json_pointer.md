# Permissive JSON Pointer 学习笔记

## Introduction

本文档解释标准 JSON Pointer 概念以及 Meilisearch 的 `permissive-json-pointer` crate 的实现差异。

---

## 📌 标准 JSON Pointer (RFC 6901)

JSON Pointer 是一个标准（RFC 6901），用于**精确定位** JSON 文档中的某个值。

### 语法规则

- 使用 `/` 作为分隔符
- 使用数组索引访问数组元素
- 有转义规则：`~0` = `~`，`~1` = `/`

### 标准示例

```json
{
  "name": "Alice",
  "pets": [
    {
      "type": "dog",
      "name": "Max"
    },
    {
      "type": "cat", 
      "name": "Luna"
    }
  ],
  "address": {
    "city": "Paris"
  }
}
```

**标准 JSON Pointer 访问：**

| Pointer | 返回值 | 说明 |
|---------|--------|------|
| `/name` | `"Alice"` | 根级别的 name |
| `/pets/0/name` | `"Max"` | 第一个宠物的名字 |
| `/pets/1/type` | `"cat"` | 第二个宠物的类型 |
| `/address/city` | `"Paris"` | 嵌套对象的值 |

**标准 JSON Pointer 特点：**
- ✅ **精确定位**单个值
- ✅ 可以访问数组的特定索引
- ✅ 路径明确，不存在歧义
- ❌ 每次只能取一个值
- ❌ 不能模糊匹配
- ❌ 不能同时访问数组所有元素

---

## 🌟 Permissive JSON Pointer 的不同

这个 crate 的设计理念完全不同，它是为 **Meilisearch 的字段选择** 场景设计的。

### 核心区别对比

| 特性 | 标准 JSON Pointer | Permissive JSON Pointer |
|------|------------------|------------------------|
| **分隔符** | `/` | `.` |
| **设计目标** | 定位单个值 | 筛选多个字段 |
| **数组访问** | 用索引 `/pets/0` | 应用到所有元素 `pets.name` |
| **冲突处理** | 不存在（路径明确） | **全部匹配**（极度宽松） |
| **返回结果** | 单个值 | 新的 JSON 对象（只含选中字段） |
| **匹配策略** | 精确匹配 | 模糊匹配（能匹配的都匹配） |

### 宽松匹配示例

对于这个"混乱"的 JSON：

```json
{
  "pet.dog.name": "jean",           // 键名本身就包含点
  "pet.dog": {
    "name": "bob"                    // 嵌套对象
  },
  "pet": {
    "dog.name": "michel",            // 另一种嵌套方式
    "dog": {
      "name": "milan"                 // 再一种嵌套方式
    }
  }
}
```

**用 `pet.dog.name` 选择器：**

- **标准 JSON Pointer**：会失败或只匹配一个（需要精确路径）
- **Permissive**：**四个都匹配！**

```json
{
  "pet.dog.name": "jean",      // ✅ 匹配（字面键名）
  "pet.dog": {
    "name": "bob"               // ✅ 匹配（嵌套对象）
  },
  "pet": {
    "dog.name": "michel",       // ✅ 匹配（混合方式）
    "dog": {
      "name": "milan"            // ✅ 匹配（完全嵌套）
    }
  }
}
```

### 数组处理的不同

**标准 JSON Pointer**（访问特定元素）：
```
/pets/0/name  → 只返回第一个宠物的名字 "Max"
```

**Permissive**（应用到所有元素）：
```rust
select_values(json, ["pets.name"])
```

给定输入：
```json
{
  "pets": [
    { "name": "Max", "age": 3 },
    { "name": "Luna", "age": 5 },
    { "type": "bird" }  // 没有 name 字段
  ]
}
```

返回结果：
```json
{
  "pets": [
    { "name": "Max" },
    { "name": "Luna" }
    // 第三个元素没有 name，所以被过滤掉了
  ]
}
```

**关键不同**：
- 标准方式需要分别访问 `/pets/0/name` 和 `/pets/1/name`
- Permissive 方式一次性处理所有数组元素

---

## 🎯 为什么设计得如此宽松？

从代码注释可以看出设计者的考虑：

```rust
// What a crappy json! But never underestimate your users, 
// they WILL somehow base their entire workflow on this kind of json.
```

**设计原因：**

1. **搜索引擎场景**：Meilisearch 需要处理用户上传的任意结构的 JSON
2. **用户友好**：用户可能不清楚字段到底是 `dog.name`（字面键名）还是嵌套的 `dog` → `name` 结构
3. **宽容性原则**：宁可多匹配也不漏掉数据（搜索引擎的核心哲学）
4. **多源数据**：同一份 JSON 可能来自不同系统，结构不统一

**哲学对比：**
- **标准 JSON Pointer**：严格、精确、程序员友好
- **Permissive JSON Pointer**：宽松、模糊、用户友好

---

## 📝 在 Meilisearch 中的实际用途

在 Meilisearch API 中使用场景：

```http
POST /indexes/movies/search
{
  "q": "action",
  "attributesToRetrieve": ["title", "cast.name", "director.info.name"]
}
```

这个 crate 确保：
- 无论用户的 JSON 结构多奇怪
- 只要"看起来像" `cast.name` 的字段
- 都会被识别并返回

**另一个用途：字段投影（Field Projection）**

```rust
// 用户上传的复杂文档
let document = json!({
    "title": "Inception",
    "cast": [
        { "name": "Leonardo", "age": 45 },
        { "name": "Tom", "age": 55 }
    ],
    "metadata": { /* 大量其他字段 */ }
});

// 只返回需要的字段
let result = select_values(&document, ["title", "cast.name"]);
// 结果只包含 title 和 cast 数组中的 name 字段
```

---

## 🔍 简单类比

理解两者区别的最佳类比：

| 标准 JSON Pointer | Permissive JSON Pointer |
|-------------------|------------------------|
| GPS 精确导航 | 模糊搜索 |
| 必须知道准确地址 | "附近的都告诉我" |
| `/pets/0/name` | `pets.name` |
| 返回一个值 | 返回筛选后的文档 |
| 用于**定位** | 用于**筛选** |

---

## 💡 学习建议

理解这个 crate 的关键：

1. **忘记精确性**：不要用标准 JSON Pointer 的思维方式
2. **拥抱模糊**：理解"能匹配的都匹配"的设计哲学
3. **看测试用例**：`lib.rs` 中的测试用例是最好的文档
4. **关注边界**：重点看 `contained_in()` 函数如何判断匹配

---

## 📚 相关资源

- [RFC 6901 - JSON Pointer 标准](https://tools.ietf.org/html/rfc6901)
- [Meilisearch 文档 - Attributes to Retrieve](https://www.meilisearch.com/docs)
- 源码：`crates/permissive-json-pointer/src/lib.rs`

## 具体的实现

```rust
pub fn select_values<'a>(
    value: &Map<String, Value>,
    selectors: impl IntoIterator<Item = &'a str>,
) -> Map<String, Value> {
    let selectors = selectors.into_iter().collect();
    create_value(value, selectors)
}
```
- 为什么 `collect()` 返回 `HashSet<&str>` 而不是 `Vec`？
  `collect()`的方法是把迭代器的元素"收集"到一个容器里。
  ```rust
  // 例子1：收集到 Vec
  let numbers = vec![1, 2, 3];
  let doubled: Vec<i32> = numbers.iter().map(|x| x * 2).collect();
  // doubled = [2, 4, 6]

  // 例子2：收集到 HashSet
  use std::collections::HashSet;
  let set: HashSet<i32> = numbers.iter().copied().collect();
  ```
  关键观察：同样是 collect()，但收集到了不同的容器！编译器可以根据返回标注的类型自动推断collect之后
  需要转换成的类型。只要返回的这个类型实现了FromIterator即可。
  这里因为create_value的第二个参数是HashSet<&str>, 而HashSet又实现了FromIterator, 所以
  collect之后的类型自动被转换为HashSet<&str>。


- `impl IntoIterator<Item = &'a str>` 详解
  关于这里的函数参数 `selectors: impl IntoIterator<Item = &'a str>` 是什么意思？
  这里的意思就是selectors必须实现IntoIterator这个trait（IntoIterator代表这个类型可以变转换成一个迭代器），其中Item就是这个
  iterator返回元素的类型，即一个&str. 
  ```rust
  pub trait IntoIterator {
    /// The type of the elements being iterated over.
    #[stable(feature = "rust1", since = "1.0.0")]
    type Item;

    /// Which kind of iterator are we turning this into?
    #[stable(feature = "rust1", since = "1.0.0")]
    type IntoIter: Iterator<Item = Self::Item>;
  ```
  如果我们需要制定Item的类型，我们使用<>在trait后面制定Item的类型。

  这个type叫做associated type, 跟generics有点像，也需要在实现的时候制定，但是跟generics的区别是，你一个struct实现一个使用associated type的trait
  只允许有一个Item类型。但是对于generics, 你可以有多个不同的类型实现。举例：
  ```rust
  pub trait Iterator {
    type Item;

    fn next(&mut self) -> Option<Self::Item>;
  }

  pub trait Iterator<T> {
    fn next(&mut self) -> Option<T>;
  }
  ```
  对于实现基于genetics的Iterator, 我们可以impl Iterator<u32> for Counter, impl Iterator<String> for Counter。
  而对于使用associated types的Iterator，我们只能指定一个type. 
  

- 为什么 `pub fn select_values` 要标记 `<'a>` （生命周期）？
  rust book中关于longest string的例子
  ```rust
  fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
  }
  ```
  这里标记'a到底是什么意思？'a表示的就是一个抽象的变量的有效的生命周期，我们把它放在了两个参数和返回值上的意思就是想说：
  在这个周期中，x, y, 返回值都必须是有效的。因为返回值要么是x, 要么是y，所以说，这里的意思就是返回值的存活的这个
  范围必须x,y都是存活的？加入x,y是不一样的，那么这里的'a就表示活的短的那个变量的生命周期，也是说，我们要求:
  **使用返回值的生命周期只能跟x,y中最短的一样**. 即然现在我们理解了为什么要这样标注，那么为什么需要这样标注呢？原因就是：
  **因为函数的返回值我们不知道要返回的x还是y，所以我们这里如果返回的是生命周期较短的变量，和我们对返回值的生命周期的要求**
  **就是只能在短的生命周期中使用，这就保证我们的返回值一定要不会应用一个已经“过期”的变量。**
  
  接下里，让我们看一个错误标准的生命周期:
  ```rust
  fn shortest<'a, 'b>(x: &'a str, y: &'b str) -> &'a str {
  if x.len() < y.len() {
    x
  } else {
    y
  }
  }
  ```
  上面这个标注生命意思，即返回的这个值，只要和x活的一样长就行了。这肯定是不行的，因为如果返回的y, 而y的生命周期更短，这就会导致我们在一个x还有效，但y已经被
  drop了的时候，试图去获取y的值。
  还有一点就是，我引用rust book中的一段:
  > Ultimately, lifetime syntax is about connecting the lifetimes of various parameters and return values of functions. Once they’re connected, Rust has enough information to allow memory-safe operations and disallow operations that would create dangling pointers or otherwise violate memory safety.

  lifetime永远是关于连接参数和返回值的，因为如果我们不连接他们，比如：
  ```rust
  fn longest<'a>(x: &str, y: &str) -> &'a str {
    let result = String::from("really long string");
    result.as_str()
  }
  ```
  返回值返回的是一个在函数内定义的变量的reference，这个是没有意义的，因为result只在函数内有效. 我们返回值的
  lifetime一定是关于函数参数的。（当然，首先需要是引用我们才谈返回值，如果是一个新创建的对象就不需要在意了）

  同样的，struct又可以定义liftime, 看下面这个例子:
  ```rust
  struct ImportantExcerpt<'a> {
    part: &'a str,
  }
  ```
  同样，我们在part这个引用上定义了一个生命周期'a, 的意思是如果我们生命一个struct，
  那么这个struct只能在part的生命周期内使用（看，这是一种约束）。
  官方文档的话: **an instance of ImportantExcerpt can’t outlive the reference it holds in its part field.**


  lifetime elision rules 生命周期省略规则
  首先介绍一个概念:
  Lifetimes on function or method parameters are called input lifetimes, and lifetimes on return values are called output lifetimes.
  lifetime elision rules是三条规则，当编译器apply这三条规则之后，如果还有没有被标注的参数/返回值，那么就需要手动标注。
  注意，lifetime elision rules只适用于function以及impl blocks中的方法。
  - 规则1: Compiler assigns a different lifetime parameter to each lifetime in each input type. 
    举个例子:
    ```rust
    fn foo(x: &i32, y: &i32) -> 自动标注 fn foo<'a, 'b>(x: &'a i32, y: &'b i32). 
    fn foo(x: &ImportantExcerpt) -> 自动标注 fn foo<'a, 'b>(x: &'a ImportantExcerpt<'b>).
    ```
    其实我们发现，如果没有返回值的话，rust的第一条规则都可以覆盖，因为lifetime是来处理paramter和return value, 即然都没有
    return value那么自然就不需要lifetime标注。

    注意：对于第二个function, 我们是需要两个生命周期的。`'a` 表示的是对于ImportantExcerpt的reference的生命周期，'b表示的是受里面的part的生命周期的约束。
    可以理解成一个是结构题的引用可以活多久，一个是part可以活多久（可以简单理解为这个结构体可以活多久）。
    如果只有一个标识符（即 &'a ImportantExcerpt<'a>），那就强制要求“对结构体的借用时间”必须等于“结构体内部数据的有效期”。
    下面是一个很好的例子为什么要decouple这两个生命周期
    ```rust
    struct ImportantExcerpt<'b> {
    part: &'b str,
    }

    // 展开后等同于：fn announce<'a, 'b>(x: &'a ImportantExcerpt<'b>)
    fn announce(x: &ImportantExcerpt) {
        println!("Attention: {}", x.part);
    }

    fn main() {
        // 1. 内部数据的生命周期 'b 开始
        let long_lived_string = String::from("This is a very long string");
        
        let excerpt = ImportantExcerpt {
            part: &long_lived_string,
        };

        {
            // 2. 我们在这里发起一个临时的借用
            // 此时，对结构体的引用 &excerpt 的生命周期是 'a
            // 'a 仅仅存在于这个花括号内
            announce(&excerpt); 
            
            // 'a 在这里结束
        }

        // 3. excerpt 和 long_lived_string 在这里依然有效
        println!("The excerpt is still here: {}", excerpt.part);
      }
  ```
  - 规则2: if there is exactly one input lifetime parameter, that lifetime is assigned to all output lifetime parameters
  ```rust
  fn foo<'a>(x: &'a i32) -> &'a i32
  ```
  这个也很好理解，如果只有一个输入，那肯定return value的lifetime只能跟它关联。
  - 规则3: if there are multiple input lifetime parameters, but one of them is &self or &mut self because this is a method, the lifetime of self is assigned to all output lifetime parameters. 
  对于method的lifetimes annotations基本就是和规则3相关，比如这里
  ```rust
  impl<'a> ImportantExcerpt<'a> {
    fn announce_and_return_part(&self, announcement: &str) -> &str {
        println!("Attention please: {announcement}");
        self.part
    }
  }
  ```
  这里我们回单的结构题里面的一个引用，那么我们的return value的lifetime必然就和part是一样的，都是'a. 

  **注意：即使我们的lifetime被figure out出来之后，也不代表能够通过编译**
  比如这个比较极端例子：
  ```rust
  struct SomeStruct<'a> {
      a: &'a str,
      b: &'a str,
  }

  impl<'a> SomeStruct<'a> {
      fn longest(&self, x: &str, y: &str) -> &str {
          if x.len() > y.len() { x } else { y }
      }
  }

  fn main() {}
  ```
  利用规则3， return value是可以被标注为&self的生命周期的，但是其实返回值的生命周期是受制于x, y的。这个时候，编译器会报错，我们还是需要手动标注。

  现在让我们来回看permissive_json_pointer中的代码。
  其实我不是很懂这里为什么必须要标记，之后可以看看有没有类似的代码比较一下，但是
  AI大致的解释就是，这里标注的意思是selectors中产生的引用是在使用函数select_values期间是有效的（我寻思这不废话吗，都传进来了还能无效吗？）
  ```rust
  pub fn select_values<'a>(
    value: &Map<String, Value>,
    selectors: impl IntoIterator<Item = &'a str>,
  ) -> Map<String, Value> {
    let selectors = selectors.into_iter().collect();
    create_value(value, selectors)
  }
  ```














