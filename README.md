## How to run

Given a configuration like, written in a file named `config.yml` in the `/configs` folder:
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
if you run:
```
CONFIG_FILE_PATH=configs/config.yml GITLAB_TOKEN=MY_TOKEN cargo run
```

the script will retrieve data from the gitlab-ci of the repository up to two tags for each and will be generated a file markdown as the following :

<table>
<tr>
<td>mongo<td>4.0<td>4.2<td>4.4<td>5.0
<tr><td>my-service@v5.2.1<td>true<td>false<td>true<td>true
<tr><td>my-service@v4.4.0<td>true<td>false<td>true<td>false
<tr><td>another-service@v5.3.2<td>true<td>false<td>true<td>true
<tr><td>another-service@v4.4.0<td>true<td>false<td>true<td>false
</table>
