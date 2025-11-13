# 静态资源目录

这个目录用于存放静态文件，如图片、CSS、JavaScript 等。

## 访问方式

静态文件可以通过以下路径访问：

```
http://localhost:8080/api/tiny-note/static/<文件路径>
```

## 示例

- `static/images/default_avatar.png` → `http://localhost:8080/api/tiny-note/static/images/default_avatar.png`

## 注意事项

1. 所有放在此目录下的文件都可以公开访问，无需认证
2. 建议为不同类型的文件创建子目录（如 images/、css/、js/）
3. 文件路径区分大小写
