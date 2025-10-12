# Rust 笔记应用后端项目生成提示词

你是一名资深的 Rust 后端工程师。
请使用 **Rust** 生成一个用于笔记应用的、可用于生产环境的完整后端项目。

## 🎯 项目目标
构建一个用于「笔记应用」的后端 API，提供：
- 基于 JWT 的用户认证与授权
- 笔记的增删改查（CRUD）
- 标签与分类管理（可选）
- 搜索与筛选功能
- 基于 Redis 的缓存
- 使用 MySQL 作为主数据库
- 模块化、整洁且可维护的架构

---

## 🧰 技术栈
- 框架：Axum（最新稳定版本）
- 数据库：MySQL（通过 SQLx）
- 缓存：Redis
- 认证：JSON Web Token（jsonwebtoken crate）
- 密码哈希：argon2
- 环境管理：dotenvy
- 日志：tracing
- 配置：dotenv + 配置文件
- 异步运行时：Tokio
- HTTP 中间件：tower-http（用于 CORS 等）

---

## 🏗️ 项目结构
请将代码组织为如下清晰的模块：
src/
├── main.rs
├── config.rs
├── db/
│   ├── mod.rs
│   ├── mysql.rs
│   └── redis.rs
├── models/
│   ├── user.rs
│   └── note.rs
├── routes/
│   ├── mod.rs
│   ├── auth.rs
│   └── notes.rs
├── services/
│   ├── auth_service.rs
│   └── note_service.rs
├── utils/
│   ├── jwt.rs
│   └── password.rs
└── middleware/
    ├── auth_middleware.rs
    └── logging.rs

---

## 🧩 核心功能

### 1️⃣ 用户系统
- 统一前缀：`/api/tiny-note`
- `POST /api/tiny-note/auth/register` → 注册新用户
- `POST /api/tiny-note/auth/login` → 使用 email+password 登录并获取 JWT
- 使用 Argon2 进行密码哈希
- 基于 JWT 的认证（Bearer Token）
- 使用 Redis 存储会话/令牌黑名单

### 2️⃣ 笔记管理
- `POST /api/tiny-note/notes` → 创建笔记
- `GET /api/tiny-note/notes` → 列出笔记（支持标签与关键字筛选）
- `GET /api/tiny-note/notes/:id` → 查看笔记
- `PUT /api/tiny-note/notes/:id` → 更新笔记
- `DELETE /api/tiny-note/notes/:id` → 删除笔记
- 每条笔记关联用户（user_id）

### 3️⃣ 中间件
- 认证中间件校验 JWT 并注入用户信息
- 错误处理中间件，返回规范化的 JSON 响应
- 跨域中间件（CORS）：使用 tower-http 进行配置
  - 允许来源：任意域名，并支持携带 Cookie 请求（通过镜像请求的 Origin）
  - 允许方法：`GET`, `POST`, `PUT`, `DELETE`, `OPTIONS`
  - 允许头：`Authorization`, `Content-Type`
  - 启用凭据：`Access-Control-Allow-Credentials: true`
  - 实现建议：`AllowOrigin::mirror_request()` + `allow_credentials(true)`
 - 请求日志中间件：记录 `method`、`path`、`query`、`Content-Type` 与响应 `status`/`reason`；在路由处理函数记录关键参数（避免记录敏感信息）。

### 4️⃣ 数据库模型
**User**
```rust
id: uuid
username: String
email: String
password_hash: String
created_at: DateTime<FixedOffset>
```
 
#### ⏱️ 时间与时区策略
- 数据库存储时间为东八区上海时间（`UTC+08:00`）。
- 写入时使用 `CONVERT_TZ(UTC_TIMESTAMP(), '+00:00', '+08:00')`。
- 读取时直接返回上海本地时间，模型为 `DateTime<FixedOffset>`，前端 JSON 显示 `+08:00` 偏移。