Tiny Note Backend（Rust、Axum）

- 框架：`axum`
- 数据库：`MySQL`（通过 `sqlx`）
- 缓存：`Redis`
- 认证：`JWT`（`jsonwebtoken`）
- 密码：`argon2`
- 配置：`.env`（通过 `dotenvy`）
- 日志：`tracing`
- 运行时：`tokio`

快速开始
- 复制 `.env.example` 到 `.env` 并填写变量。
- 创建 MySQL 表（见下方示例）。
- 启动 Redis：`redis-server`。
- 构建并运行：`cargo run`。
- 可选：设置 `PORT`（默认 `8080`）。示例（demo）：`PORT=8081 cargo run --bin demo`。

配置（.env）
- `DATABASE_URL`：MySQL DSN，例如 `mysql://user:password@localhost:3306/tiny_notes`
- `REDIS_URL`：Redis 连接地址，例如 `redis://127.0.0.1:6379`
- `JWT_SECRET`：用于签发 JWT 的密钥
- `PORT`：服务端口（可选，默认 `8080`）

API
 - 统一前缀：`/api/tiny-note`
 - POST `/api/tiny-note/auth/register` { username, email, password }
 - POST `/api/tiny-note/auth/login` { email, password } -> { token }
 - 笔记接口（需要 `Authorization: Bearer <token>`）：
   - POST `/api/tiny-note/notes`
   - GET `/api/tiny-note/notes`（查询参数：`tag`, `q`）
   - GET `/api/tiny-note/notes/:id`
   - PUT `/api/tiny-note/notes/:id`
   - DELETE `/api/tiny-note/notes/:id`

CORS 与 Cookie
- CORS 镜像请求的 `Origin` 并启用凭据（`Access-Control-Allow-Credentials: true`）。
- 若要跨域携带 Cookie，前端必须启用凭据。
- Fetch 示例（携带 Cookie）：
```ts
fetch('http://localhost:8081/api/tiny-note/notes', {
  method: 'GET',
  credentials: 'include', // 携带 Cookie
  headers: {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json'
  }
}).then(res => res.json());
```
- Axios 示例（携带 Cookie）：
```ts
import axios from 'axios';

const api = axios.create({
  baseURL: 'http://localhost:8081',
  withCredentials: true, // 携带 Cookie
});

api.get('/api/tiny-note/notes', {
  headers: { Authorization: `Bearer ${token}` },
});
```
- 说明：
  - 服务端返回 `Access-Control-Allow-Credentials: true` 且镜像 `Origin`（不是 `*`）。
  - 若服务端在跨站场景设置 Cookie，需使用 `SameSite=None; Secure`，以满足现代浏览器要求。

MySQL 表结构
```sql
CREATE TABLE IF NOT EXISTS users (
  id            BINARY(16)      NOT NULL,
  username      VARCHAR(64)     NOT NULL UNIQUE,
  email         VARCHAR(128)    NOT NULL UNIQUE,
  password_hash VARCHAR(255)    NOT NULL,
  created_at    DATETIME        NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS notes (
  id         BINARY(16)   NOT NULL,
  user_id    BINARY(16)   NOT NULL,
  title      VARCHAR(255) NOT NULL,
  content    TEXT         NOT NULL,
  tags       VARCHAR(255) NULL,
  created_at DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  INDEX idx_notes_user (user_id),
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
```

说明
- SQLx 在此使用动态查询以避免编译期数据库检查。
- Redis 将令牌黑名单存储在 `bl:<jti>` 键下，并设置 TTL。
- 可在 `db/mysql.rs` 中调整连接池大小。
 - 请求日志：默认启用 `tracing`，记录每次请求与响应。
   - 请求：`method`、`path`、`query`、`Content-Type`
   - 响应：`status`、`reason`
   - 路由中记录关键参数（如 `username`、`email`、`user_id`、`note id`、`title` 等），不记录敏感信息（如密码）。
 - 时间与时区：
   - 存储策略为东八区（上海时间，`UTC+08:00`）。
   - 写入 `created_at/updated_at` 使用 `CONVERT_TZ(UTC_TIMESTAMP(), '+00:00', '+08:00')` 保证按上海时区入库。
   - 读取时直接返回上海时区时间，模型使用 `DateTime<FixedOffset>`；前端 JSON 显示为 `+08:00` 偏移的本地时间。
    - 如需在数据库层面默认使用上海时区，可设置 MySQL `time_zone = '+08:00'` 或使用触发器实现。