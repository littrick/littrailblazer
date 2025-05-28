```sh
docker run -it --rm --name ubuntu \
-v $(realpath .):/ws -w /ws \
ubuntu:22.04
```