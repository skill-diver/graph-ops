import MinusCircleOutlined from "@ant-design/icons/lib/icons/MinusCircleOutlined";
import PlusOutlined from "@ant-design/icons/lib/icons/PlusOutlined";
import {
  Button,
  Card,
  Cascader,
  Divider,
  Form,
  Input,
  Modal,
  Select,
  Space,
  Typography,
} from "antd";
import React, { useState } from "react";

const { Text } = Typography;

export default function GraphDatasetForm(props: { onFinish: () => void }) {
  const [form] = Form.useForm();

  const [entityFields, setEntityFields] = useState<
    {
      label: string;
      value: string;
      children: { label: string; value: string }[];
    }[]
  >([]);
  const [edgeEntities, setEdgeEntities] = useState<
    { label: string; value: string }[]
  >([]);
  const [loaded, setLoaded] = useState(false);

  const fetchFields = async () => {
    const fields_url = import.meta.env.VITE_OFNIL_BACKEND_URL + "/fields";
    const response: Map<string, { name: string; entity_id: string }> = await (
      await fetch(fields_url)
    ).json();
    return response;
  };

  const fetchEntities = async () => {
    const entities_url = import.meta.env.VITE_OFNIL_BACKEND_URL + "/entities";
    const response: Map<
      string,
      Map<string, { name: string; tlabel: string }>
    > = await (await fetch(entities_url)).json();
    return response;
  };

  // load Data
  (() => {
    if (!loaded) {
      setLoaded(true);

      fetchEntities().then((response) => {
        const edge_entities: { label: string; value: string }[] = [];
        Object.entries(response).map(([key, value]) => {
          if (Object.keys(value)[0] == "Edge") {
            edge_entities.push({ label: key.split("/")[2], value: key });
          }
        });

        setEdgeEntities(edge_entities);
      });

      fetchFields().then((response) => {
        const entity_fields = new Map<string, string[]>();

        Object.entries(response).map(([key, value]) => {
          if (entity_fields.get(value.entity_id) == undefined) {
            entity_fields.set(value.entity_id, [key]);
          } else {
            entity_fields.get(value.entity_id)?.push(key);
          }
        });

        const entity_fields_options = Array.from(entity_fields.entries()).map(
          ([key, value]) => {
            return {
              label: key.split("/")[2],
              value: key,
              children: value.map((field_id) => ({
                value: field_id,
                label: field_id.split("/")[3],
              })),
            };
          }
        );
        setEntityFields(entity_fields_options);
      });
    }
  })();

  const onFinish = () => {
    console.log("form", form.getFieldsValue());

    const form_values = form.getFieldsValue();

    // TO DISCUSS: here the whole table feature view object
    const default_table_feature_view = {
      name: "fraud_detection_train_user_features",
      variant: { Default: [] },
      entity_id: "default/Entity/neo4j_reviewer",
      field_ids: [
        // "default/Field/neo4j_reviewer/aggregate_neighbors_1_price",
        // "default/Field/neo4j_reviewer/aggregate_neighbors_5_rank1",
        // "default/Field/neo4j_reviewer/aggregate_neighbors_5_rank2",
        // "default/Field/neo4j_reviewer/page_rank_2",
        // "default/Field/neo4j_reviewer/betweenness_centrality_3",
        // "default/Field/neo4j_reviewer/triangle_count_4",
      ],
      online: false,
      description: null,
      created_at: null,
      updated_at: null,
      tags: {},
      owner: null,
      rendering_opt: { output_type: "NdArray", mode: "PythonBinding" },
    };

    const default_topology_feature_view = {
      name: "fraud_detection_train_topo",
      variant: { Default: [] },
      topology_type: "AdjacencyMatrix",
      online: false,
      topology_ids: [
        // "default/Topology/neo4j_sameRates",
        // "default/Topology/neo4j_rates",
      ],
      description: null,
      created_at: null,
      updated_at: null,
      tags: {},
      owners: [],
      rendering_opt: {
        layout: "CompressedSparseRow",
        mode: "PythonBinding",
      },
    };

    const default_graph_dataset = {
      name: "fraud_detection_train_dataset",
      variant: { Default: [] },
      description: null,
      table_feature_views: [default_table_feature_view],
      topology_feature_views: [default_topology_feature_view],
      rendering_opt: { sampling: null },
      deployed: false,
    };

    const table_feature_views = form_values.table_feature_views
      ? form_values.table_feature_views.map(
          (fv: { name: string; fields: [string, string][] }) => ({
            ...default_table_feature_view,
            name: fv.name,
            entity_id: fv.fields[0][0],
            field_ids: fv.fields.map((field) => field[1]),
          })
        )
      : [];

    // TODO: the topology id is edge entities ids for now (not topology ids). We may discuss the Topology struct later.
    const topology_feature_views = form_values.topology_feature_views
      ? form_values.topology_feature_views.map(
          (fv: {
            name: string;
            topology_type: string;
            edge_entities: string[];
          }) => ({
            ...default_topology_feature_view,
            name: fv.name,
            topology_type: fv.topology_type,
            topology_ids: fv.edge_entities,
          })
        )
      : [];

    const post_graph_dataset = {
      ...default_graph_dataset,
      name: form_values.dataset_name,
      table_feature_views: table_feature_views,
      topology_feature_views: topology_feature_views,
    };

    console.log(
      "posting!",
      post_graph_dataset,
      table_feature_views,
      topology_feature_views
    );

    const post_graph_dataset_url =
      import.meta.env.VITE_OFNIL_BACKEND_URL + "/graph_dataset";
    const post_table_feature_view_url =
      import.meta.env.VITE_OFNIL_BACKEND_URL + "/table_feature_view";
    const post_topology_feature_view_url =
      import.meta.env.VITE_OFNIL_BACKEND_URL + "/topology_feature_view";
    // const post_topology_url =
    //   import.meta.env.VITE_OFNIL_BACKEND_URL + "/topology";

    let num_posted_table_fvs = 0;
    // let num_posted_topologies = 0;
    let num_posted_topology_fvs = 0;
    let num_posted_graph_datasets = 0;
    for (const fv of table_feature_views) {
      fetch(post_table_feature_view_url, {
        method: "POST",
        body: JSON.stringify(fv),
      }).then((response) => {
        if (response.status == 200) {
          num_posted_table_fvs++;
          console.log("Success:", response);
        } else {
          console.error("Post Table FV Error:", response);
        }
      });
    }

    for (const fv of topology_feature_views) {
      fetch(post_topology_feature_view_url, {
        method: "POST",
        body: JSON.stringify(fv),
      }).then((response) => {
        if (response.status == 200) {
          num_posted_topology_fvs++;
          console.log("Success:", response);
        } else {
          console.error("Post Topology FV Error:", response);
        }
      });
    }

    fetch(post_graph_dataset_url, {
      method: "POST",
      body: JSON.stringify(post_graph_dataset),
    }).then((response) => {
      if (response.status == 200) {
        num_posted_graph_datasets++;
        console.log("Success:", response);
        Modal.success({
          title: "Success",
          content: "Graph dataset is successfully registered",
        });
        props.onFinish();
      } else {
        console.error("Post Graph Dataset Error:", response);
      }
    });

    setTimeout(() => {
      console.log(
        "Graph dataset is successfully registered, with" +
          num_posted_table_fvs +
          " TableFeatureView(s), " +
          num_posted_topology_fvs +
          " TopologyFeatureView(s), " +
          num_posted_graph_datasets +
          " GraphDataset(s)."
      );
    }, 2000);
  };

  return (
    <Form form={form} onFinish={onFinish} layout="vertical">
      <Form.Item
        name="dataset_name"
        rules={[
          {
            required: true,
            message: "Please input Dataset name or delete this Dataset.",
          },
        ]}
        label={<Text type="secondary">Dataset name</Text>}
      >
        <Input placeholder="Dataset Name" allowClear />
      </Form.Item>
      <Form.List name="table_feature_views">
        {(fvs, { add, remove }) => (
          <Space.Compact direction="vertical" block style={{ width: "100%" }}>
            {fvs.map(({ key, name }) => {
              return (
                <Space.Compact key={key} block>
                  <Card style={{ width: "95%" }}>
                    <Form.Item
                      name={[name, "name"]}
                      rules={[
                        {
                          required: true,
                          message:
                            "Please input TableFeatureView name or delete this TableFeatureView.",
                        },
                      ]}
                      label={
                        <Text type="secondary">TableFeatureView name</Text>
                      }
                    >
                      <Input placeholder="TableFeatureView name" allowClear />
                    </Form.Item>

                    <Form.Item
                      name={[name, "fields"]}
                      validateTrigger={["onChange", "onBlur"]}
                      label={
                        <Text type="secondary">
                          Select Fields (must have the same Entity)
                        </Text>
                      }
                      rules={[
                        {
                          required: true,
                          message: "At least one Field is required.",
                        },
                        {
                          message: "Selected Fields must have the same Entity",
                          validator: (_, value: [string, string][]) => {
                            if (value.length < 2) return Promise.resolve();
                            const entity_id = value[0][0];
                            for (let i = 1; i < value.length; i++) {
                              if (value[i][0] != entity_id) {
                                return Promise.reject("different entity");
                              }
                            }
                            return Promise.resolve();
                          },
                        },
                      ]}
                    >
                      <Cascader
                        showSearch
                        allowClear
                        multiple
                        placeholder="Search to Select Fields"
                        options={entityFields}
                      />
                    </Form.Item>
                  </Card>
                  <MinusCircleOutlined
                    onClick={() => remove(name)}
                    style={{ width: "5%" }}
                  />
                </Space.Compact>
              );
            })}
            <Form.Item>
              <Button
                type="dashed"
                onClick={() => add()}
                icon={<PlusOutlined />}
                block
              >
                Add TableFeatureView
              </Button>
            </Form.Item>
          </Space.Compact>
        )}
      </Form.List>
      <Divider />
      <Form.List name="topology_feature_views">
        {(fvs, { add, remove }) => (
          <Space.Compact direction="vertical" block style={{ width: "100%" }}>
            {fvs.map(({ key, name }) => (
              <Space.Compact key={key} block>
                <Card style={{ width: "95%" }}>
                  <Form.Item
                    name={[name, "name"]}
                    rules={[
                      {
                        required: true,
                        message:
                          "Please input TopologyFeatureView name or delete this  TopologyFeatureView.",
                      },
                    ]}
                    label={
                      <Text type="secondary">TopologyFeatureView name</Text>
                    }
                  >
                    <Input placeholder="TopologyFeatureView name" allowClear />
                  </Form.Item>
                  <Form.Item
                    name={[name, "edge_entities"]}
                    label={<Text type="secondary">Select Edge Entities</Text>}
                  >
                    <Select
                      showSearch
                      allowClear
                      mode="multiple"
                      placeholder="Search to Select Edge Entities"
                      optionFilterProp="children"
                      filterOption={(input, option) =>
                        (option?.label ?? "").includes(input)
                      }
                      filterSort={(optionA, optionB) =>
                        (optionA?.label ?? "")
                          .toLowerCase()
                          .localeCompare((optionB?.label ?? "").toLowerCase())
                      }
                      options={edgeEntities}
                    />
                  </Form.Item>
                  <Form.Item
                    name={[name, "topology_type"]}
                    label={<Text type="secondary">Select Topology Type</Text>}
                  >
                    <Select
                      showSearch
                      allowClear
                      placeholder="Search to Select Topology Type"
                      optionFilterProp="children"
                      filterOption={(input, option) =>
                        (option?.label ?? "").includes(input)
                      }
                      filterSort={(optionA, optionB) =>
                        (optionA?.label ?? "")
                          .toLowerCase()
                          .localeCompare((optionB?.label ?? "").toLowerCase())
                      }
                      options={[
                        { value: "AdjacencyList", label: "AdjacencyList" },
                        { value: "AdjacencyMatrix", label: "AdjacencyMatrix" },
                        {
                          value: "BipartiteGraphChain",
                          label: "BipartiteGraphChain",
                        },
                      ]}
                    />
                  </Form.Item>
                </Card>
                <MinusCircleOutlined
                  onClick={() => remove(name)}
                  style={{ width: "5%" }}
                />
              </Space.Compact>
            ))}
            <Form.Item>
              <Button
                type="dashed"
                onClick={() => add()}
                icon={<PlusOutlined />}
                block
              >
                Add TopologyFeatureView
              </Button>
            </Form.Item>
          </Space.Compact>
        )}
      </Form.List>
      <Divider />
      <Form.Item>
        <Button type="primary" htmlType="submit" block>
          Submit
        </Button>
      </Form.Item>
    </Form>
  );
}
