import ConfigProvider from "antd/es/config-provider";
import Layout from "antd/es/layout";
import React from "react";
import { useState } from "react";
import { Outlet, Route, Routes, useLocation } from "react-router-dom";

import GraphDatasets from "./components/GraphDatasets";
import GraphSources from "./components/GraphSources";
import InfraConfigs from "./components/InfraConfigs";
import Pipelines from "./components/Pipelines";
import SideBar, { Page } from "./components/SideBar";
import Workflows from "./components/Workflows";

const { Sider } = Layout;

function PageLayout() {
  const [collapsed, setCollapsed] = useState(true);
  const location = useLocation();
  return (
    <ConfigProvider
      theme={{
        token: {
          colorPrimary: "#535bf2",
        },
      }}
    >
      <Layout style={{ minHeight: "100vh" }}>
        <Sider
          id="sidebar"
          collapsible
          collapsed={collapsed}
          onCollapse={(value) => setCollapsed(value)}
        >
          {/* hardcode account info now */}
          <SideBar
            user="ofnil"
            project="Ofnil Demo"
            active={location.pathname.substring(1)}
          />
        </Sider>
        <Layout>
          <Outlet />
        </Layout>
      </Layout>
    </ConfigProvider>
  );
}

export default function App() {
  return (
    <Routes>
      <Route path="/" element={<PageLayout />}>
        <Route index element={<GraphSources />} />
        <Route path={Page.Sources} element={<GraphSources />} />
        <Route path={Page.Pipelines} element={<Pipelines />} />
        <Route path={Page.Workflows} element={<Workflows />} />
        <Route path={Page.Datasets} element={<GraphDatasets />} />
        <Route path={Page.Infras} element={<InfraConfigs />} />
      </Route>
    </Routes>
  );
}
