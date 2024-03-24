import Button from "antd/es/button";
import type { ColumnsType, TableProps } from "antd/es/table";
import React, { useState } from "react";

import RegisterGraphSourceModal from "./GraphSources/GraphSourceModal";
import TableView, { TableBuilder, compareString } from "./TableView";

interface DataType {
  key: string; // resource_id
  graph: {
    sink_infra_id: object;
    database?: string;
  };
}

class GraphSourcesTableBuilder implements TableBuilder<DataType> {
  sourceSet = new Set<string>();

  getColumns(): ColumnsType<DataType> {
    return [
      {
        title: "ID",
        dataIndex: "key",
        defaultSortOrder: "descend",
        sorter: (a: DataType, b: DataType) => compareString(a.key, b.key),
      },
      {
        title: "SOURCE",
        dataIndex: "source",
        onFilter: (value: string | number | boolean, record: DataType) => {
          return (
            Object.keys(record.graph.sink_infra_id)[0].indexOf(
              value.toString()
            ) == 0
          );
        },
        filters: Array.from(this.sourceSet).map((source) => {
          return { text: source, value: source };
        }),
        render: (_, record) => (
          <Button
            type="link"
            onClick={(event: object) => showInfraConfig(record, event)}
          >
            {Object.keys(record.graph.sink_infra_id)[0] +
              ": " +
              Object.values(record.graph.sink_infra_id)[0]}
          </Button>
        ),
      },
      {
        title: "DATABASE",
        dataIndex: "database",
        render: (_, record) => {
          if (record.graph.database) {
            return record.graph.database;
          } else {
            return "-";
          }
        },
      },
      {
        title: "DETAILS",
        dataIndex: "details",
        render: (_, record) => (
          <Button
            type="link"
            onClick={(event: object) => showGraphSource(record, event)}
          >
            View
          </Button>
        ),
      },
    ];
  }

  processData(data: object): DataType[] {
    if (data == null) {
      return [];
    }
    return Object.entries(data).map((record) => {
      this.sourceSet.add(Object.keys(record[1].sink_infra_id)[0]);
      return {
        key: record[0],
        graph: record[1],
      };
    });
  }
}

function showGraphSource(data: DataType, event: object) {
  console.log(data, event);
  // TODO(tatiana)
}

function showInfraConfig(data: object, event: object) {
  console.log(data, event);
  // TODO(tatiana)
}

// for debug
const onChange: TableProps<DataType>["onChange"] = (
  pagination,
  filters,
  sorter,
  extra
) => {
  console.log("params", pagination, filters, sorter, extra);
};

export default function GraphSources() {
  const builder = new GraphSourcesTableBuilder();
  const [updateTimestamp, setUpdateTimestamp] = useState<number>();
  return (
    <TableView
      title={<b>GRAPH SOURCES</b>}
      builder={builder}
      onChange={onChange}
      path="graphs"
      updateTimestamp={updateTimestamp}
    >
      <RegisterGraphSourceModal
        onFinish={() => setUpdateTimestamp(Date.now())}
      />
    </TableView>
  );
}
