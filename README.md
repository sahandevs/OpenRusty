# [WIP] OpenRusty
budget OpenResty written in rust.


# ngnix.conf
```

events { }

http {
    server {
        listen 8082;

        location / {
            open_rusty_request_filter 'for h in headers.keys() { if h == "Drop-Me" { return 403; } }';
        }
    }
}
```


## Manual test

```
curl -i -H "Test-Header3: YES" http://localhost:8082/
```

## Rapid dev with nginx

```
(sudo ./objs/nginx -s stop || echo "nginx is not running") && cargo build --release --manifest-path=$MODULE_PATH/Cargo.toml && rm "./objs/ngx_modules.o" && rm ./objs/nginx && make && sudo ./objs/nginx
```