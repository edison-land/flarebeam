# FlareBeam

本地开发服务，扫码就能在手机上打开。未配对的请求到不了 localhost。

[官网](https://flarebeam.zwteam.top) · [下载](https://github.com/edison-land/flarebeam/releases/latest)

## 安装

仅支持 macOS（Apple Silicon 和 Intel 通用）。

1. 安装 cloudflared：`brew install cloudflared`
2. 从 [Releases](https://github.com/edison-land/flarebeam/releases/latest) 下载 `FlareBeam_<版本>_universal.dmg`，把 FlareBeam 拖进「应用程序」。
3. 应用没有签名，第一次打开前运行：

   ```sh
   xattr -cr /Applications/FlareBeam.app
   ```

## 使用

选端口 → 开启预览 → 手机扫码 → 输入电脑上显示的 4 位 PIN。

## 工作原理

```
手机 → Cloudflare Quick Tunnel → FlareBeam 代理（127.0.0.1:随机端口）→ localhost:<端口>
```

- 隧道指向 FlareBeam 代理，不指向你的开发端口。
- 没有会话 Cookie 的请求，代理直接返回 401 和 PIN 页，不会转发。
- PIN 正确后下发 HttpOnly 会话 Cookie，4 小时有效。
- 连续输错 5 次，PIN 验证锁定 5 分钟。
- 二维码里只有隧道地址，PIN 只显示在电脑上。
- 选「公开访问」可以跳过 PIN。

## 开发

需要 Node 22+、pnpm 9 和 Rust stable。

```sh
pnpm install
pnpm tauri dev
```

落地页在 `landing/`：

```sh
cd landing
pnpm install
pnpm dev
```

## 发布

- **桌面端**：把 `src-tauri/tauri.conf.json` 的 `version` 改成新版本，推送同名标签（如 `v0.1.0`）。GitHub Actions 会构建通用版 dmg 并发布到 Releases。
- **落地页**：`landing/` 的改动合入 `main` 后，自动部署到 [flarebeam.zwteam.top](https://flarebeam.zwteam.top)。需要仓库 secrets `CLOUDFLARE_API_TOKEN` 和 `CLOUDFLARE_ACCOUNT_ID`。

## 许可证

[MIT](LICENSE)
