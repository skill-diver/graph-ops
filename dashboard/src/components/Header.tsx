import Layout from "antd/es/layout";
import theme from "antd/es/theme";
import React, { ReactNode } from "react";

import "./Header/Header.css";

export default function Header(props: {
  title: ReactNode;
  children: ReactNode;
}) {
  const {
    token: { colorBgContainer },
  } = theme.useToken();
  return (
    <Layout.Header
      id="header-nav"
      style={{
        background: colorBgContainer,
        verticalAlign: "middle",
      }}
    >
      <div className="header-nav-item">{props.title}</div>
      <div className="header-nav-item header-nav-right">{props.children}</div>
    </Layout.Header>
  );
}
