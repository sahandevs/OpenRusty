# [WIP] OpenRusty
budget OpenResty written in rust.


### nginx.conf
```
events { }

http {
        server {
                listen 8082;

                location /with_mod {
                        open_rusty_request_filter 'for h in headers.keys() { }';
                }

                location / {

                }
        }
}
```


### Manual test

```
curl -i -H "Drop-Me: YES" http://localhost:8082/
```

### Benchmark

nginx.conf:

```
events { }

http {
        server {
                listen 8082;

                location /with_mod {
                        open_rusty_request_filter 'for h in headers.keys() { }';
                        index with_mod.html;
                        root html;

                }

                location / {
                        root html;
                        index index.html index.htm;
                }
        }
}
```

benchmark result using [wrk](https://github.com/wg/wrk)

```sh
# No module
$ wrk -t12 -c400 -d30s http://localhost:8082/index.html

Running 30s test @ http://localhost:8082/index.html
  12 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency    91.58ms  186.44ms   2.00s    95.08%
    Req/Sec   410.10    194.19     1.26k    66.68%
  138116 requests in 30.05s, 111.95MB read
  Socket errors: connect 0, read 219038, write 0, timeout 559
Requests/sec:   4595.97
Transfer/sec:      3.73MB

# With module
$ wrk -t12 -c400 -d30s http://localhost:8082/with_mod.html

Running 30s test @ http://localhost:8082/with_mod.html
  12 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency    96.11ms  180.16ms   1.99s    95.42%
    Req/Sec   393.82    175.11     1.50k    68.31%
  131956 requests in 30.10s, 106.96MB read
  Socket errors: connect 0, read 208911, write 0, timeout 495
Requests/sec:   4383.87
Transfer/sec:      3.55MB
```

> Memory usage for Nginx stays at ~1MB throughout the benchmark for both.

> nginx 1.19.3 on Ubuntu on wsl1

### Rapid dev

```
(sudo ./objs/nginx -s stop || echo "nginx is not running") && cargo build --release --manifest-path=$MODULE_PATH/Cargo.toml && ((rm "./objs/ngx_modules.o" && rm ./objs/nginx) || echo "no artifacts") && make && echo "========make done===========" && sudo ./objs/nginx
```