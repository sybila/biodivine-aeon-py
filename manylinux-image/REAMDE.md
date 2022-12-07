Because we need `libclang` and `z3` during build, we can't use the default manylinux docker. Here, we define a docker image that derives from manylinux but actually has the dependencies to build Z3.

To publish the docker image, run:

```
docker build -t daemontus/manylinux-aeon .
docker push daemontus/manylinux-aeon
```

( If this is a new person maintaining this: use your own username and then change it in the CI script ;) )