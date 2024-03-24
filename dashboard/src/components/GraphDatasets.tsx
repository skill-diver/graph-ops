import CheckCircleFilled from "@ant-design/icons/CheckCircleFilled";
import { Spin } from "antd";
import Button from "antd/es/button";
import Collapse from "antd/es/collapse";
import type { ColumnsType } from "antd/es/table";
import Tag from "antd/es/tag";
import theme from "antd/es/theme";
import React, { useState } from "react";

import GraphDatasetModal from "./GraphDatasets/GraphDatasetModal";
import TableView, { TableBuilder, compareString } from "./TableView";

interface DataType {
  key: string; // resource id
  dataset: {
    description: string;
    deployed: boolean;
    table_feature_views: {
      name: string;
      entity_id: string;
      field_ids: string[];
      rendering_opt?: { output_type: string };
    }[];
    topology_feature_views: {
      name: string;
      topology_ids: string[];
      rendering_opt?: { layout: string };
    }[];
    rendering_opt?: {
      sampling?: object;
    };
  };
}

// hardcode now, no deployment yet
function DeploymentStatus(props: {
  deploy: boolean;
  colorTextSecondary: string;
  colorPrimary: string;
}) {
  const [deployment, setDeployment] = useState<0 | 1 | 2>(props.deploy ? 1 : 0); // no, yes, processing

  if (deployment == 1) {
    return <CheckCircleFilled style={{ color: props.colorPrimary }} />;
  }
  if (deployment == 2) {
    return <Spin style={{ color: props.colorTextSecondary }} />;
  }
  return (
    <Button
      type="primary"
      onClick={() => {
        setDeployment(2);
        setTimeout(() => {
          setDeployment(1);
        }, 10000);
      }}
    >
      Deploy
    </Button>
  );
}

class GraphDatasetsTableBuilder implements TableBuilder<DataType> {
  colorTextSecondary: string;
  colorPrimary: string;

  constructor(colorTextSecondary: string, colorPrimary: string) {
    this.colorTextSecondary = colorTextSecondary;
    this.colorPrimary = colorPrimary;
  }

  getColumns(): ColumnsType<DataType> {
    return [
      {
        title: "ID",
        dataIndex: "key",
        defaultSortOrder: "descend",
        sorter: (a: DataType, b: DataType) => compareString(a.key, b.key),
        render: (resource_id) => (
          <Button
            type="link"
            onClick={(event) => showGraphDataset(resource_id, event)}
          >
            {resource_id}
          </Button>
        ),
      },
      {
        title: "DEPLOYMENT",
        dataIndex: "deployment",
        align: "center",
        render: (_, record) => {
          return (
            <DeploymentStatus
              deploy={record.dataset.deployed}
              colorTextSecondary={this.colorTextSecondary}
              colorPrimary={this.colorPrimary}
            />
          );
        },
      },
      {
        title: "Table Feature Views",
        dataIndex: "tablefv",
        responsive: ["xl"],
        render: (_, record: DataType) => {
          return (
            <Collapse
              size="small"
              expandIconPosition="start"
              style={{ width: "fit-content" }}
            >
              {record.dataset.table_feature_views.map((view) => (
                <Collapse.Panel
                  showArrow={false}
                  key={view.name}
                  header={
                    <div
                      className="flex"
                      style={{ justifyContent: "space-between" }}
                    >
                      <span
                        style={{
                          marginRight: "1em",
                          overflowWrap: "anywhere",
                        }}
                      >
                        {view.name}
                      </span>
                      <span style={{ textAlign: "right" }}>
                        <Tag>{view.rendering_opt?.output_type}</Tag>
                        <Tag
                          style={{
                            backgroundColor: this.colorPrimary,
                            opacity: 0.85,
                            color: "white",
                          }}
                        >
                          {view.entity_id}
                        </Tag>
                      </span>
                    </div>
                  }
                >
                  {view.field_ids.map((field_id, index) => (
                    <Button
                      key={index}
                      type="link"
                      style={{ display: "block" }}
                    >
                      {field_id}
                    </Button>
                  ))}
                </Collapse.Panel>
              ))}
            </Collapse>
          );
        },
      },
      {
        title: "Topology Feature Views",
        dataIndex: "topologyfv",
        responsive: ["xl"],
        render: (_, record: DataType) => (
          <Collapse size="small" expandIconPosition="start">
            {record.dataset.topology_feature_views.map((view) => (
              <Collapse.Panel
                key={view.name}
                header={
                  <div
                    className="flex"
                    style={{ justifyContent: "space-between" }}
                  >
                    <span
                      style={{ marginRight: "1em", overflowWrap: "anywhere" }}
                    >
                      {view.name}
                    </span>
                    <span>
                      <Tag>{view.rendering_opt?.layout}</Tag>
                    </span>
                  </div>
                }
                showArrow={false}
              >
                {view.topology_ids.map((topo_id, index) => (
                  <Button key={index} type="link" style={{ display: "block" }}>
                    {topo_id}
                  </Button>
                ))}
              </Collapse.Panel>
            ))}
          </Collapse>
        ),
      },
      {
        title: "Preprocessing",
        dataIndex: "rendering_opt",
        responsive: ["xxl"],
        render: (_, record: DataType) => {
          if (record.dataset.rendering_opt?.sampling) {
            return Object.keys(record.dataset.rendering_opt?.sampling);
          }
          return "N/A";
        },
      },
      {
        title: "Description",
        dataIndex: "description",
        responsive: ["xxl"],
        render: (_, record) => <p>{record.dataset.description}</p>,
      },
    ];
  }

  processData(data: object): DataType[] {
    if (data == null) {
      return [];
    }
    return Object.entries(data).map((record) => {
      return {
        key: record[0],
        dataset: record[1],
      };
    });
  }
}

function showGraphDataset(
  resource_id: string,
  event:
    | React.MouseEvent<HTMLAnchorElement, MouseEvent>
    | React.MouseEvent<HTMLButtonElement, MouseEvent>
) {
  console.log(resource_id, event);
}

export default function GraphDatasets() {
  const {
    token: { colorTextSecondary, colorPrimary },
  } = theme.useToken();
  const builder = new GraphDatasetsTableBuilder(
    colorTextSecondary,
    colorPrimary
  );
  const [updateTimestamp, setUpdateTimestamp] = useState<number>();
  return (
    <TableView
      title={<b>GRAPH SERVING</b>}
      builder={builder}
      path="graph_datasets"
      updateTimestamp={updateTimestamp}
    >
      <GraphDatasetModal onFinish={() => setUpdateTimestamp(Date.now())} />
    </TableView>
  );
}
