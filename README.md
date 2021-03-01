# [WIP] OpenRusty
budget OpenResty written in rust.


### ngnix.conf
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


### Manual test

```
curl -i -H "Drop-Me: YES" http://localhost:8082/
```

### Rapid dev

```
(sudo ./objs/nginx -s stop || echo "nginx is not running") && cargo build --release --manifest-path=$MODULE_PATH/Cargo.toml && ((rm "./objs/ngx_modules.o" && rm ./objs/nginx) || echo "no artifact") && make && echo "========make done===========" && sudo ./objs/nginx
```