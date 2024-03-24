import Avatar from "antd/es/avatar";
import Menu, { MenuProps } from "antd/es/menu";
import theme from "antd/es/theme";
import React, { ReactNode } from "react";
import { Link } from "react-router-dom";

import "./SideBar/SideBar.css";

const GraphSourcesIcon = () => (
  <span className="anticon material-symbols-outlined">database</span>
);
const WorkflowsIcon = () => (
  <span className="anticon material-symbols-outlined">code_blocks</span>
);
const GraphDatasetsIcon = () => (
  <span className="anticon material-symbols-outlined">dataset</span>
);
const InfraConfigsIcon = () => (
  <span className="anticon material-symbols-outlined">data_object</span>
);
// const PipelineLibraryIcon = () => ( <span className="anticon material-symbols-outlined">flowsheet</span>);

export enum Page {
  Sources = "sources",
  Workflows = "workflows",
  Datasets = "datasets",
  Pipelines = "pipelines",
  Infras = "infras",
}

type MenuItem = Required<MenuProps>["items"][number];
function createItem(
  key: React.Key,
  label: ReactNode,
  icon: ReactNode,
  children?: MenuItem[]
): MenuItem {
  return {
    key,
    icon,
    children,
    label: <Link to={`/${key}`}>{label}</Link>,
  } as MenuItem;
}

export default function SideBar(props: {
  user: string;
  project: ReactNode;
  active: string;
}) {
  const {
    token: { colorBgContainer, colorText },
  } = theme.useToken();
  return (
    <Menu
      theme="dark"
      defaultSelectedKeys={[props.active]}
      mode="inline"
      items={[
        createItem(Page.Sources, "Graph Sources", <GraphSourcesIcon />),
        createItem(Page.Workflows, "Workflows", <WorkflowsIcon />),
        createItem(Page.Datasets, "Graph Serving", <GraphDatasetsIcon />),
        // TODO(tatiana): leave for future plan
        // createItem(Page.Pipelines, "Pipeline Library", <PipelineLibraryIcon />),
        createItem(Page.Infras, "Infra Configs", <InfraConfigsIcon />),
        {
          style: {
            position: "absolute",
            bottom: "48px",
            verticalAlign: "middle",
          },
          key: "user",
          icon: (
            <Avatar style={{ background: colorBgContainer, color: colorText }}>
              {props.user.toUpperCase()}
            </Avatar>
          ),
          label: <a href="/account">{props.project}</a>,
        } as MenuItem,
      ]}
    />
  );
}
