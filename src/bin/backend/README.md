# Ofnil Backend API Server

```bash
cargo run -F dashboard --bin backend
```

## API

```bash
GET /entity?id=default/Entity/neo4j_reviewer
GET /entities
```

```bash
curl -XPOST http://localhost:8000/entity --data '{"Edge":{"name":"neo4j_alsoBuy","tlabel":"alsoBuy","src_tlabel":"Product","dst_tlabel":"Product","src_entity_id":"default/Entity/neo4j_product","dst_entity_id":"default/Entity/neo4j_product","directed":false,"primary_key":null,"variant":{"Default":[]}}}'
```

```bash
GET http://localhost:8000/fields
GET http://localhost:8000/fields?entity_name=neo4j_product
```

```bash
curl  http://localhost:9888/provider/entities?infra_name=neo4j_1
curl -XPOST 'http://localhost:9888/provider/fields?infra_name=neo4j_1' -d '{"Vertex":{"name":"neo4j_product","tlabel":"Product","primary_key":"asin","variant":{"Default":[]}}}'
```
