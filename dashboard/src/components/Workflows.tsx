import CheckCircleFilled from "@ant-design/icons/CheckCircleFilled";
import CheckCircleOutlined from "@ant-design/icons/CheckCircleOutlined";
import { Drawer, message } from "antd";
import Button from "antd/es/button";
import Collapse from "antd/es/collapse";
import List from "antd/es/list";
import type { ColumnsType } from "antd/es/table";
import theme from "antd/es/theme";
import Typography from "antd/es/typography";
import React, { useState } from "react";

import TableView, { TableBuilder, compareString } from "./TableView";
import RegisterWorkflow from "./Workflows/RegisterWorkflow";
import WorkFlow from "./Workflows/Workflow";
import WorkflowEditor from "./Workflows/WorkflowEditor";
import "./Workflows/Workflows.css";

const { Text } = Typography;

interface DataType {
  key: string; // resource_id
  workflow: WorkFlow;
  jobs: { id: string; body: { status: boolean } }[] | null;
}

class WorkflowsTableBuilder implements TableBuilder<DataType> {
  colorTextSecondary: string;
  colorPrimary: string;
  onClickWorkflow: (workflowId: string) => void;

  constructor(
    colorTextSecondary: string,
    colorPrimary: string,
    onClickWorkflow: (workflowId: string) => void
  ) {
    this.colorTextSecondary = colorTextSecondary;
    this.colorPrimary = colorPrimary;
    this.onClickWorkflow = onClickWorkflow;
  }

  getColumns(): ColumnsType<DataType> {
    return [
      {
        title: "ID",
        dataIndex: "key",
        defaultSortOrder: "descend",
        sorter: (a: DataType, b: DataType) => compareString(a.key, b.key),
        render: (resource_id) => (
          <Button type="link" onClick={() => this.onClickWorkflow(resource_id)}>
            {resource_id}
          </Button>
        ),
      },
      {
        title: "Output Data",
        dataIndex: "export",
        render: (_, record) => {
          const len = record.workflow.export_resources.length;
          if (len > 2) {
            const width = Math.max(
              ...record.workflow.export_resources.map(
                (export_item) => export_item[1].length
              )
            );
            return (
              <Collapse size="small" expandIconPosition="end">
                <Collapse.Panel
                  key="1"
                  header={`${len} Items`}
                  style={{ minWidth: `${width}ch` }}
                >
                  <List>
                    {record.workflow.export_resources.map((export_item) => (
                      <List.Item key={export_item[1]}>
                        <Button
                          type="link"
                          onClick={(event) =>
                            showOutputResource(export_item[1], event)
                          }
                        >
                          {export_item[1]}
                        </Button>
                      </List.Item>
                    ))}
                  </List>
                </Collapse.Panel>
              </Collapse>
            );
          } else {
            return (
              <>
                {record.workflow.export_resources.map((export_item) => (
                  <List.Item key={export_item[1]}>
                    <Button type="link">{export_item[1]}</Button>
                  </List.Item>
                ))}
              </>
            );
          }
        },
      },
      {
        title: "Job Instances",
        dataIndex: "jobs",
        render: (jobs: { id: string; body: { status: boolean } }[]) => {
          if (jobs == null) return "";
          return (
            <>
              {jobs.map((job) => {
                let avatar = (
                  <CheckCircleOutlined
                    style={{ color: this.colorTextSecondary }}
                  />
                );
                if (job.body.status) {
                  avatar = (
                    <CheckCircleFilled style={{ color: this.colorPrimary }} />
                  );
                }
                return (
                  <Button type="link" key={job.id}>
                    {avatar} {job.id}
                  </Button>
                );
              })}
            </>
          );
        },
      },
      {
        title: "Created By",
        dataIndex: "owner",
        render: (_, record) => record.workflow.owners.join(", "),
      },
      {
        title: "Description",
        dataIndex: "description",
        render: (_, record) => <p>{record.workflow.description}</p>,
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
        workflow: record[1],
        jobs: [
          // { id: `Job${Date.now()}`, body: { status: true } },
        ],
      };
    });
  }
}

function showOutputResource(
  resource_id: string,
  event:
    | React.MouseEvent<HTMLAnchorElement, MouseEvent>
    | React.MouseEvent<HTMLButtonElement, MouseEvent>
) {
  console.log(resource_id, event);
}

export default function Workflows() {
  const {
    token: { colorTextSecondary, colorPrimary },
  } = theme.useToken();
  const [workflowView, setWorkFlowView] = useState<string>("");
  const [updateTimestamp, setUpdateTimestamp] = useState<number>();
  const builder = new WorkflowsTableBuilder(
    colorTextSecondary,
    colorPrimary,
    setWorkFlowView
  );
  return (
    <>
      <TableView
        title={<b>WORKFLOWS</b>}
        builder={builder}
        path="transformations"
        updateTimestamp={updateTimestamp}
      >
        <RegisterWorkflow onFinish={() => setUpdateTimestamp(Date.now())} />
      </TableView>

      <Drawer
        className="workflow-drawer"
        width={"calc( 100% - 80px )"}
        open={workflowView.length > 0}
        title={
          <div className="flex">
            <span>
              Edit Workflow <Text type="secondary">{workflowView}</Text>
            </span>
          </div>
        }
        onClose={() => setWorkFlowView("")}
      >
        <WorkflowEditor
          workflowId={workflowView}
          onFinish={(workflowName) => {
            message.info("Saved workflow " + workflowName);
            setWorkFlowView("");
            // force table to refetch
            setUpdateTimestamp(Date.now());
          }}
        />
      </Drawer>
    </>
  );
}
