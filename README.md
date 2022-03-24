Given a configuration like, written in a file named config.yml:
```yaml
gitlab_base_api_host: "https://MY_GITLAB_API_HOST.com"
services:
  - name: my-service
    ci: 
      project_id: "platform%2Fmy-group%2Fmy-service"
    matrix: 
     - path: test.parallel.matrix[0].MONGO_VERSION
       name: mongo
  - name: another-service
    ci: 
      project_id: "platform%2Fgroup%2Fanother-service"
    matrix: 
     - path: test.parallel.matrix[0].MONGO_IMAGE
       name: mongo
```

with a gitlab-ci associated pipeline like this, retrieved by api call:

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
Service another-service
Name: mongo, Compatible: ["4.0", "4.4", "5.0"]
```