import type { ColumnsType } from "antd/es/table";
import React from "react";

import TableView, { TableBuilder } from "./TableView";

interface DataType {
  key: string; // infra id name
  infra: string; // infra type
  uri: string;
}

class InfraConfigsTableBuilder implements TableBuilder<DataType> {
  getColumns(): ColumnsType<DataType> {
    return [
      {
        title: "System",
        dataIndex: "infra",
      },
      {
        title: "Identifier",
        dataIndex: "key",
      },
      {
        title: "URI",
        dataIndex: "uri",
      },
    ];
  }
  processData(data: object): DataType[] {
    if (data == null) {
      return [];
    }
    return Object.values(data).map(
      (record: { 0: Map<string, string>; 1: string }) => {
        return {
          infra: Object.keys(record[0])[0],
          key: Object.values(record[0])[0],
          uri: record[1],
        };
      }
    );
  }
}

export default function InfraConfigs() {
  const builder = new InfraConfigsTableBuilder();
  return (
    <TableView title={<b>INFRA CONFIGS</b>} builder={builder} path="infras" />
  );
}
