import {
  Button,
  Form,
  Modal,
  Progress,
  Result,
  Select,
  Space,
  Typography,
} from "antd";
import React, { useState } from "react";

import { GraphSourceMeta } from "../GraphSourceModal";

const { Text } = Typography;

// TODO(han): add a trigger to refresh the Table
// TODO(han): refactor
export default function NextForm(props: {
  setIndex: (setIndex: number) => void;
  setIsModalOpen: (isModalOpen: boolean) => void;
  sourceMeta: GraphSourceMeta;
  onFinish: () => void;
}) {
  const sourceMeta = props.sourceMeta;

  const [form] = Form.useForm();
  const [showProgress, setShowProgress] = useState(false);
  const [graphPosted, setGraphPosted] = useState(false);

  const [entities, setEntities] = useState<
    { string: { name: string; tlabel: string } }[]
  >([]);

  const [fields, setFields] = useState<Map<string, { name: string }[]>>(
    new Map()
  );

  const [numEntities, setNumEntities] = useState(0);
  const [numFields, setNumFields] = useState(0);
  const [totalFields, setTotalFields] = useState(0);

  const postEntities = async () => {
    const post_entity_url = import.meta.env.VITE_OFNIL_BACKEND_URL + "/entity";
    let num = 0;
    const get_entity_ids = new Map();

    for await (const entity of entities) {
      const response = await fetch(post_entity_url, {
        method: "POST",
        body: JSON.stringify(entity),
      });
      if (response.status == 200) {
        num++;
        setNumEntities(num);
        const e_id = await response.text();
        get_entity_ids.set(Object.values(entity)[0].tlabel, e_id);
      } else {
        console.log("error", response);
      }
    }

    let entity_ids = {};
    for (const [key, value] of get_entity_ids) {
      entity_ids = {
        ...entity_ids,
        [key]: value,
      };
    }
    console.log("graph entity_ids", entity_ids, get_entity_ids);

    // post graph
    const post_graph_url = import.meta.env.VITE_OFNIL_BACKEND_URL + "/graph";
    const graph = {
      // example json of a Graph
      ...{
        name: "neo4j",
        variant: {
          // Default: []
        },
        description: null,
        entity_ids: {
          // isSimilarTo: "default/Entity/neo4j_isSimilarTo/neo4j_product/neo4j_product",
          // Product: "default/Entity/neo4j_product",
        },
        tags: {},
        owners: [],
        sink_infra_id: {
          // Neo4j: "neo4j_1"
        },
      },
      name: props.sourceMeta.name,
      entity_ids: entity_ids,
      sink_infra_id: sourceMeta.data_source_type
        ? {
            [sourceMeta.data_source_type]: sourceMeta.data_source_name,
          }
        : null,
      variant:
        sourceMeta.variant == "default"
          ? { Default: [] }
          : { UserDefined: sourceMeta.variant },
    };
    console.log("posting graph", graph);
    fetch(post_graph_url, {
      method: "POST",
      body: JSON.stringify(graph),
    }).then((response) => {
      if (response.status == 200) {
        console.log("graph posted");
        setGraphPosted(true);
      } else {
        console.log("post graph error", response);
      }
    });
    return;
  };

  const postFields = async () => {
    const post_field_url = import.meta.env.VITE_OFNIL_BACKEND_URL + "/field";
    let num = 0;
    console.log("postFields", fields);
    for (const fs of fields.values()) {
      console.log("postFields fs", fs);
      for (const field of fs) {
        console.log("post field", field);
        fetch(post_field_url, {
          method: "POST",
          body: JSON.stringify(field),
        }).then((response) => {
          if (response.status == 200) {
            num++;
            setNumFields(num);
          } else {
            console.log("error", response);
          }
        });
      }
    }
    return;
  };

  const onFinish = () => {
    console.log("next form", form.getFieldsValue());

    setNumEntities(0);
    setNumFields(0);
    setShowProgress(true);

    postEntities();
    postFields();
  };

  const onBack = () => {
    props.setIndex(0);
  };

  const fetchEntities = async () => {
    const url =
      import.meta.env.VITE_OFNIL_BACKEND_URL +
      "/provider/entities" +
      (sourceMeta.data_source_name
        ? "?infra_name=" + sourceMeta.data_source_name
        : "");
    const response: { string: { name: string; tlabel: string } }[] = await (
      await fetch(url)
    ).json();
    console.log("fetch entities", response);
    if (props.sourceMeta.name) {
      for (const e of response) {
        Object.values(e)[0].name =
          props.sourceMeta.name + "_" + Object.values(e)[0].name;
      }
    }
    console.log("modified entities", response);
    return response;
  };

  const entityKey = (entity: { string: { name: string; tlabel: string } }) =>
    Object.keys(entity)[0] + "/" + Object.values(entity)[0].name;

  const fetchFields = async (
    es: { string: { name: string; tlabel: string } }[]
  ) => {
    const provider_fields_url =
      import.meta.env.VITE_OFNIL_BACKEND_URL +
      "/provider/fields" +
      (sourceMeta.data_source_name
        ? "?infra_name=" + sourceMeta.data_source_name
        : "");

    const fields_map = new Map<
      string,
      { name: string; value_type: string; entity_id: string }[]
    >();

    let total_fields = 0;

    console.log("fetch fields, es:", es);
    for (const entity of es) {
      console.log(entity);
      const entity_key = entityKey(entity);
      const res = await (
        await fetch(provider_fields_url, {
          method: "POST",
          body: JSON.stringify(entity),
        })
      ).json();
      console.log("fetch fields", res);
      fields_map.set(entity_key, res);
      total_fields += res.length;
    }
    setTotalFields(total_fields);
    return fields_map;
  };

  // Load Data
  (() => {
    console.log("sourceMeta", props.sourceMeta);

    if (entities.length == 0) {
      fetchEntities().then((response) => {
        setEntities(Array.from(response));

        if (fields.size == 0) {
          fetchFields(response).then((fields_map) => {
            console.log("fields_map", fields_map);
            setFields(new Map(fields_map));
          });
        }
      });
    }
  })();

  return (
    <>
      <Text type="warning" italic>
        Identify Primary Key for Entities:
      </Text>
      <Form form={form} onFinish={onFinish} layout="vertical">
        {(() => {
          return entities
            .map(entityKey)
            .sort()
            .reverse()
            .map((entity_key) => (
              <Form.Item
                name={entity_key}
                key={entity_key}
                label={<Text type="secondary">{entity_key}</Text>}
              >
                <Select>
                  {fields.get(entity_key)?.map((field) => (
                    <Select.Option key={field.name} value={field.name}>
                      {field.name}
                    </Select.Option>
                  ))}
                </Select>
              </Form.Item>
            ));
        })()}
        <Form.Item>
          <Button onClick={onBack} block>
            Back
          </Button>
        </Form.Item>

        <Form.Item>
          <Button type="primary" htmlType="submit" block>
            Submit
          </Button>
        </Form.Item>
      </Form>
      <RegisterProgress
        showProgress={showProgress}
        setShowProgress={setShowProgress}
        numEntities={numEntities}
        numFields={numFields}
        totalEntities={entities.length}
        totalFields={totalFields}
        setIndex={props.setIndex}
        setIsModalOpen={props.setIsModalOpen}
        setEntities={setEntities}
        setFields={setFields}
        graphPosted={graphPosted}
        setGraphPosted={setGraphPosted}
        onFinish={props.onFinish}
      />
    </>
  );
}

function RegisterProgress(props: {
  showProgress: boolean;
  setShowProgress: (show: boolean) => void;
  numEntities: number;
  numFields: number;
  totalEntities: number;
  totalFields: number;
  setIndex: (setIndex: number) => void;
  setIsModalOpen: (isModalOpen: boolean) => void;
  setEntities: (
    entities: { string: { name: string; tlabel: string } }[]
  ) => void;
  setFields: (fields: Map<string, { name: string }[]>) => void;

  graphPosted: boolean;
  setGraphPosted: (graphPosted: boolean) => void;
  onFinish: () => void;
}) {
  const {
    showProgress,
    setShowProgress,
    numEntities,
    numFields,
    totalEntities,
    totalFields,
    setIndex,
    setIsModalOpen,
    setEntities,
    setFields,
    graphPosted,
    setGraphPosted,
  } = props;

  return (
    <Modal
      open={showProgress}
      onCancel={() => {
        setShowProgress(false);
      }}
      onOk={() => {
        setShowProgress(false);
      }}
      footer={null}
    >
      <Text>
        Register Entities: ({numEntities} / {totalEntities})
      </Text>
      <Space.Compact block>
        <Progress percent={(numEntities / totalEntities) * 100}></Progress>
      </Space.Compact>
      <Text>
        Register Fields: ({numFields} / {totalFields})
      </Text>
      <Space.Compact block>
        <Progress percent={(numFields / totalFields) * 100}></Progress>
      </Space.Compact>
      <Result
        status={graphPosted ? "success" : "info"}
        title={graphPosted ? "Graph Registered!" : "Posting Graph..."}
        subTitle={`(${numEntities} / ${totalEntities}) entities and (${numFields} / ${totalFields}) fields registered.`}
        extra={
          <Button
            onClick={() => {
              setShowProgress(false);
              setIsModalOpen(false);
              props.onFinish();
              setIndex(0);
              setEntities([]);
              setFields(new Map());
              setGraphPosted(false);
            }}
          >
            OK!
          </Button>
        }
      />
    </Modal>
  );
}
