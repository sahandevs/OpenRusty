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

### Benchmark

nginx.conf:

```
error_log logs/error.log debug;

events { }

http {
        server {
                listen 8082;

                location /with_mod {
                        open_rusty_request_filter 'for h in headers.keys() { return 403; }';
                }

                location / {

                }
        }
}
```

benchmark result using [wrk](https://github.com/wg/wrk)

```sh
# No module
$ wrk -t12 -c400 -d30s http://localhost:8082/

Running 30s test @ http://localhost:8082/
  12 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency    74.60ms  149.89ms   2.00s    97.09%
    Req/Sec   409.09    189.86     1.53k    67.53%
  131744 requests in 30.10s, 106.79MB read
  Socket errors: connect 0, read 234632, write 0, timeout 694
Requests/sec:   4376.96
Transfer/sec:      3.55MB

# With module
$ wrk -t12 -c400 -d30s http://localhost:8082/with_mod
Running 30s test @ http://localhost:8082/with_mod
  12 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency   138.73ms  298.08ms   1.92s    88.47%
    Req/Sec     1.00k   344.37     2.10k    66.76%
  351910 requests in 30.10s, 103.35MB read
  Socket errors: connect 0, read 126829, write 0, timeout 0
  Non-2xx or 3xx responses: 351910
Requests/sec:  11692.61
Transfer/sec:      3.43MB
```

> Memory usage for Nginx stays at ~1MB throughout the benchmark for both.

> nginx 1.19.3 on Ubuntu on wsl1

### Rapid dev

```
(sudo ./objs/nginx -s stop || echo "nginx is not running") && cargo build --release --manifest-path=$MODULE_PATH/Cargo.toml && ((rm "./objs/ngx_modules.o" && rm ./objs/nginx) || echo "no artifacts") && make && echo "========make done===========" && sudo ./objs/nginx
```