import { Divider, Form, Input, Modal, Select, Typography } from "antd";
import { Store } from "antd/es/form/interface";
import type { ModalProps } from "antd/es/modal";
import React, { ReactNode, useEffect, useState } from "react";

const { Text } = Typography;

function varToTitle(value: string): string {
  return value.toUpperCase().replaceAll("_", " ");
}

function getInput(
  value_key: string,
  value_type: string, //"string" | "number" | "string[]" | "select",
  value: object,
  updateInitialValues: (key: string, value: object | string) => void
) {
  if (value != null) {
    updateInitialValues(value_key, value);
  }
  if (value_type == "string" || typeof value == "string") {
    return <Input allowClear placeholder={value != null ? "" : "Optional"} />;
  }
  if (value_type == "number" || typeof value == "number") {
    return (
      <Input type="number" placeholder={value != null ? "" : "Optional"} />
    );
  }
  throw new Error("unexpected type " + value_type + ". key: " + value_key);
}

export type FormState =
  | {
      values: Store;
      configs: Config | null;
    }
  | undefined;
type ConfigModalProps = {
  onSave: (key: string, value: FormState) => void;
  onCancel: ModalProps["onCancel"];
  target: string; // node id
  title: string;
  open: boolean;
  formState: FormState;
};
export default function ConfigModal({
  target,
  title,
  onSave,
  onCancel,
  open,
  formState,
}: ConfigModalProps) {
  const [configs, setConfigs] = useState<Config | null>(null);
  const [formContent, setFormContent] = useState<ReactNode>();
  const [form] = Form.useForm();

  const reinitForm = (config: Config) => {
    form.resetFields();
    const defaultValues: Store = {};
    const content = createForm(config, (k, v) => {
      defaultValues[k] = v;
    });
    setConfigs(config);
    setFormContent(content);
    // initialValues={formContent?.values} does not work
    if (formState?.values) {
      form.setFieldsValue(formState.values);
    } else {
      form.setFieldsValue(defaultValues);
    }
  };

  // update form state on target change
  useEffect(() => {
    // get default configs from target node id (.../<procedure>) by GET /configs/<procedure>
    const procedure = target.substring(target.lastIndexOf("/") + 1);
    if (procedure.length == 0) return;

    if (formState?.configs) {
      reinitForm(formState.configs);
    } else {
      // fetch configs if null
      fetch(import.meta.env.VITE_OFNIL_BACKEND_URL + "/configs/" + procedure)
        .then((response) => response.json())
        .then((data) => {
          reinitForm(data);
        });
    }
  }, [target]);

  return (
    <Modal
      maskClosable={false}
      title={title}
      open={open}
      onOk={() => {
        form
          .validateFields()
          .then((values) => {
            onSave(target, { values, configs: configs as Config });
          })
          .catch((info) => {
            form.scrollToField(info.errorFields[0].name[0]);
          });
      }}
      onCancel={onCancel}
      okText="Save"
    >
      <Form form={form} layout="vertical">
        {formContent}
      </Form>
    </Modal>
  );
}

interface ConfigItem {
  input_type: "select" | "multiple" | "multiselect";
  key: string;
  value: object;
}
interface Config {
  [x: string]: ConfigItem | ConfigItem[] | Config;
}

function getFormItemInput(
  item: ConfigItem,
  updateInitialValues: (key: string, value: object | string) => void
) {
  const values = item.value as string[];
  const options = values.map((val) => {
    let valstr = val.toString();
    if (typeof val != "string") {
      valstr = Object.entries(val)
        .map((val) => val.join(":"))
        .toString();
    }
    return {
      key: valstr,
      value: typeof val == "string" ? val : JSON.stringify(val),
      label: valstr,
    };
  });
  switch (item.input_type) {
    case "select": {
      updateInitialValues(item.key, options[0].value);
      return <Select defaultActiveFirstOption showSearch options={options} />;
    }
    case "multiple": {
      // TODO(tatiana): require inputs of the number values.length
      // Select may not be a good choice, any component for a group of inputs?
      return (
        <Select
          allowClear
          mode="tags"
          options={options}
          placeholder={`e.g. ${options[0].value}`}
        />
      );
    }
    case "multiselect": {
      return <Select allowClear mode="multiple" options={options} />;
    }
  }
}
function createFormItem(
  item: ConfigItem,
  updateInitialValues: (key: string, value: object | string) => void,
  key_prefix: string
): ReactNode {
  const key_splits = item.key.split(".");
  const input = getFormItemInput(item, updateInitialValues);
  return (
    <Form.Item
      key={key_prefix + item.key}
      name={key_prefix + item.key}
      rules={[{ required: item.value != null }]}
      label={
        <Text type="secondary" style={{ fontSize: 12 }}>
          {varToTitle(key_splits[key_splits.length - 1])}
        </Text>
      }
    >
      {input}
    </Form.Item>
  );
}

// TODO(tatiana): explicitly handle different types of configs and add related onFieldsChange logic
function createForm(
  configs: Config,
  updateInitialValues: (key: string, value: object | string) => void
): ReactNode {
  configs = Object.values(configs)[0] as Config;
  const content = Object.entries(configs).map(([key, entry]) => {
    if (entry instanceof Array<ConfigItem>) {
      return (
        <div key={key}>
          <Divider orientation="left" orientationMargin={0}>
            <Text style={{ fontSize: 14 }}>{varToTitle(key)}</Text>
          </Divider>
          {entry.map((item) =>
            createFormItem(item, updateInitialValues, key + ".")
          )}
        </div>
      );
    }
    if (entry.input_type) {
      return createFormItem(entry as ConfigItem, updateInitialValues, "");
    }

    // nested Config
    entry = Object.values(entry)[0];
    return (
      <div key={key}>
        <Divider orientation="left" orientationMargin={0}>
          <Text style={{ fontSize: 14 }}>{varToTitle(key)}</Text>
        </Divider>
        {Object.entries(entry).map(([item_key, item]) => {
          const key_splits = item_key.split(",", 2);
          const value_type = key_splits[1];
          const value_key = key + "." + item_key;
          return (
            <Form.Item
              key={value_key}
              name={value_key}
              rules={[{ required: item != null }]}
              label={
                <Text type="secondary" style={{ fontSize: 12 }}>
                  {varToTitle(key_splits[0])}
                </Text>
              }
            >
              {getInput(value_key, value_type, item, updateInitialValues)}
            </Form.Item>
          );
        })}
      </div>
    );
  });
  return <>{content}</>;
}
