import Layout from "antd/es/layout";
import Table from "antd/es/table";
import type { ColumnsType, TableProps } from "antd/es/table";
import React, { ReactNode } from "react";

import Header from "./Header";

export function compareString(a: string, b: string): number {
  if (a < b) {
    return -1;
  } else if (a == b) {
    return 0;
  } else {
    return 1;
  }
}

export interface TableBuilder<DataType> {
  getColumns(): ColumnsType<DataType>;
  processData(data: object): DataType[];
}

export default class TableView<
  DataType extends object
> extends React.Component<{
  title: ReactNode;
  builder: TableBuilder<DataType>;
  onChange?: TableProps<DataType>["onChange"];
  children?: ReactNode;
  path: string;
  updateTimestamp?: number;
}> {
  constructor(props: {
    title: ReactNode;
    builder: TableBuilder<DataType>;
    onChange?: TableProps<DataType>["onChange"];
    children?: ReactNode;
    path: string;
  }) {
    super(props);
  }

  fetchData() {
    fetch(import.meta.env.VITE_OFNIL_BACKEND_URL + "/" + this.props.path)
      .then((response) => {
        if (!response.ok) throw new Error(response.status.toString());
        response.json().then((data) => {
          this.setState(data);
        });
      })
      .catch((error) => console.log(error));
  }

  componentDidUpdate(prevProps: { updateTimestamp?: number }) {
    if (this.props.updateTimestamp != prevProps.updateTimestamp) {
      this.fetchData();
    }
  }

  componentDidMount(): void {
    this.fetchData();
  }

  render() {
    return (
      <>
        <Header title={this.props.title}>{this.props.children}</Header>
        <Layout.Content
          id="contents"
          style={{
            marginTop: 16,
            marginBottom: 16,
            marginLeft: 34,
            marginRight: 50,
          }}
        >
          <Table
            tableLayout="auto"
            size="middle"
            rowSelection={{ type: "checkbox" }}
            dataSource={this.props.builder.processData(this.state)}
            columns={this.props.builder.getColumns()}
            onChange={this.props.onChange}
            pagination={{
              hideOnSinglePage: true,
              defaultPageSize: 25,
              showSizeChanger: true,
            }}
          />
        </Layout.Content>
      </>
    );
  }
}
