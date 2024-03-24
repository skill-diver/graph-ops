import Button from "antd/es/button";
import React, { ReactNode } from "react";

export default function DragProcedureButton(props: {
  id: string;
  label: ReactNode;
  upstream: string;
  nodeType: string;
}) {
  return (
    <Button
      style={{ display: "block", width: "100%" }}
      draggable
      onDragStart={(event) => {
        event.dataTransfer.effectAllowed = "move";
        event.dataTransfer.setData("app/reactflow", props.nodeType);
        // procedure name
        event.dataTransfer.setData("app/reactflow/key", props.id);
        // upstream
        event.dataTransfer.setData("app/reactflow/upstream", props.upstream);
        // position and width of node to create
        const bound = event.currentTarget.getBoundingClientRect();
        event.dataTransfer.setData(
          "app/reactflow/x",
          (event.clientX - bound.left).toString()
        );
        event.dataTransfer.setData(
          "app/reactflow/y",
          (event.clientY - bound.top).toString()
        );
        event.dataTransfer.setData(
          "app/reactflow/width",
          bound.width.toString()
        );
      }}
    >
      {props.label}
    </Button>
  );
}
