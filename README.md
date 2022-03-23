Given a service like:
```yaml
name: my-service
matrix:
    - path: "test.parallel.matrix[0].MONGO_VERSION"
      name: mongo
```

with a gitlab-ci associated pipeline like:

```yaml
install-dependencies:
  before_script:
    - npm version
test:
  parallel:
    matrix:
      - MONGO_HOST_CI: 'mongo:27017'
        MONGO_VERSION: [\"4.0\", \"4.4\", \"5.0\"]

test-latest:
  variables:
    MONGO_HOST_CI: 'mongo:27017'

  services:
    - name: mongo
      alias: mongo
```    

will be returned:

```
Service my-service
Name: mongo, Compatible: ["4.0", "4.4", "5.0"]
```