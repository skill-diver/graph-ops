import { Button, Form, Input, Select, Tag, Typography } from "antd";
import React, { useState } from "react";

import { GraphSourceMeta } from "../GraphSourceModal";

const { Text } = Typography;

export default function FirstForm(props: {
  setIndex: (setIndex: number) => void;
  setSourceMeta: (sourceMeta: GraphSourceMeta) => void;
}) {
  const [form] = Form.useForm();
  const [dataSources, setDataSources] = useState<Map<string, string>[]>([]);
  const [databases, setDatabases] = useState<string[]>([]);
  const [loaded, setLoaded] = useState(false);

  const onFinish = () => {
    console.log("form", form.getFieldsValue());
    props.setSourceMeta({
      data_source_type: form.getFieldValue("data_source").split(" ||| ")[0],
      data_source_name: form.getFieldValue("data_source").split(" ||| ")[1],
      database: form.getFieldValue("database"),
      name: form.getFieldValue("name"),
      variant: form.getFieldValue("variant"),
    });

    props.setIndex(1);
  };

  const fetchAllDataSources = async () => {
    console.log("fetching all data sources");
    const url = import.meta.env.VITE_OFNIL_BACKEND_URL + "/infras";
    if (!loaded) {
      const response: { 0: Map<string, string>; 1: string }[] = await (
        await fetch(url)
      ).json();
      console.log(response);

      const data_sources: Map<string, string>[] = [];

      for (const ds of response) {
        const infra_type = Object.keys(ds[0])[0];
        const infra_name = Object.values(ds[0])[0];
        data_sources.push(new Map().set(infra_type, infra_name));
      }
      setLoaded(true);
      setDataSources(data_sources);
      return response;
    }
  };

  fetchAllDataSources();

  const fetchDatabases = async (dataSource: Map<string, string>) => {
    // TODO: this is hard-code for this neo4j instance
    console.log("fetching databases from data source", dataSource);
    const all_dbs = ["demo"].sort();
    setDatabases(all_dbs);
  };

  const onDataSourceChange = (value: Map<string, string>) => {
    console.log(`selected ${value}`);
    if (value) {
      fetchDatabases(value);
    }
  };

  return (
    <Form form={form} onFinish={onFinish} layout="vertical">
      <Form.Item
        name="data_source"
        rules={[
          {
            required: true,
            message: "Please select data source.",
          },
        ]}
        label={
          <Text type="secondary" style={{ fontSize: 12 }}>
            Data Source
          </Text>
        }
      >
        <Select
          placeholder="Data Source"
          onChange={onDataSourceChange}
          allowClear
        >
          {dataSources.map((ds) => {
            const infra_type = ds.keys().next().value;
            const infra_name = ds.values().next().value;

            return (
              <Select.Option
                key={infra_type + infra_name}
                value={`${infra_type} ||| ${infra_name}`} // special split str: ' ||| '
              >
                <Text>
                  <Tag>{infra_type}</Tag> {infra_name}
                </Text>
              </Select.Option>
            );
          })}
        </Select>
      </Form.Item>
      <Form.Item
        name="database"
        label={
          <Text type="secondary" style={{ fontSize: 12 }}>
            Database Name
          </Text>
        }
      >
        <Select placeholder="Database Name" allowClear>
          {databases.map((database) => (
            <Select.Option key={database} value={database}>
              {database}
            </Select.Option>
          ))}
        </Select>
      </Form.Item>
      <Form.Item
        name="name"
        rules={[
          {
            required: true,
            message: "Please specify a name.",
          },
        ]}
        label={
          <Text type="secondary" style={{ fontSize: 12 }}>
            Save As Name (will be used as Entity prefix)
          </Text>
        }
      >
        <Input placeholder="Save As Name" allowClear />
      </Form.Item>
      <Form.Item
        name="variant"
        initialValue="default"
        label={
          <Text type="secondary" style={{ fontSize: 12 }}>
            Variant
          </Text>
        }
      >
        <Input placeholder="Variant" allowClear />
      </Form.Item>
      <Form.Item>
        <Button type="primary" htmlType="submit" block>
          Next
        </Button>
      </Form.Item>
    </Form>
  );
}
