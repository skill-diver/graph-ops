import Input from "antd/es/input";
import List from "antd/es/list";
import hljs from "highlight.js";
import "highlight.js/styles/github.css";
import React, { ReactNode, useRef } from "react";
import { Handle, Position } from "reactflow";
import { NodeToolbar } from "reactflow";

import Cypher from "./Cypher";
import DragProcedureButton from "./DragProcedureButton";

const defaultToolbarPosition = Position.Bottom;
hljs.registerLanguage("cypher", Cypher);

export type ProcedureList = {
  key: string;
  label: ReactNode;
  loading?: boolean;
  // specifies nodeType according to the exported nodeTypes
  outputNodeType: string;
}[];

export interface SourceNodeProps {
  toolbarPosition?: Position;
  id: string;
  data: {
    label: ReactNode;
    procedureList: ProcedureList;
  };
}

function SourceNode({ toolbarPosition, data, id }: SourceNodeProps) {
  const nodeStyle = {
    border: "2px solid #535bf2",
    background: "#535bf2",
    color: "white",
    borderRadius: 5,
  };

  if (toolbarPosition == undefined) {
    toolbarPosition = defaultToolbarPosition;
  }
  return (
    <div style={nodeStyle}>
      <NodeToolbar position={toolbarPosition}>
        <List
          className="nopan"
          // bordered style={{ borderColor: "var(--theme-purple)", padding: "10px" }}
          itemLayout="vertical"
          loading={data.procedureList.length == 0}
          dataSource={data.procedureList}
          renderItem={(item) => (
            <DragProcedureButton
              nodeType={item.outputNodeType}
              id={item.key}
              label={item.label}
              upstream={id}
            />
          )}
        />
      </NodeToolbar>
      <div style={{ padding: "10px 20px" }}>
        <div style={{ color: "white", opacity: 0.8 }}>
          <i>Source</i>
        </div>
        {data.label}
      </div>
      {/* <Handle type="source" position={Position.Top} id="t" />
      <Handle type="source" position={Position.Bottom} id="b" /> */}
      <Handle type="source" position={Position.Right} id="r" />
    </div>
  );
}

export interface ProcedureNodeProps {
  id: string;
  toolbarPosition?: Position;
  data: {
    isQuery: boolean;
    label: ReactNode;
    procedureList?: ProcedureList;
    width: number;
    upstream: string;
  };
}
// TODO(tatiana): delete downstreams on delete?
function ProcedureNode({ toolbarPosition, data, id }: ProcedureNodeProps) {
  if (data.procedureList == undefined) {
    data.procedureList = [];
  }
  if (toolbarPosition == undefined) {
    toolbarPosition = defaultToolbarPosition;
  }
  const markup = useRef<HTMLDivElement>(null);
  return (
    <div
      style={{
        border: "1px solid #535bf2",
        background: "white",
        borderRadius: 5,
        textAlign: "center",
        minWidth: data.isQuery ? 400 : data.width,
      }}
    >
      <NodeToolbar position={toolbarPosition}>
        <List
          // bordered style={{ borderColor: "var(--theme-purple)", padding: "10px" }}
          itemLayout="vertical"
          loading={data.procedureList.length == 0}
          dataSource={data.procedureList}
          renderItem={(item) => (
            <DragProcedureButton
              nodeType={item.outputNodeType}
              id={item.key}
              label={item.label}
              upstream={id}
            />
          )}
        />
      </NodeToolbar>
      <div style={{ padding: "4px 15px" }}>{data.label}</div>
      {data.isQuery ? (
        <div style={{ position: "relative" }} className="nodrag">
          <Input.TextArea
            className="markup-textarea ant-input"
            color="transparent"
            style={{ backgroundColor: "transparent", width: "100%" }}
            onChange={(event) => {
              if (markup.current) {
                markup.current.innerHTML = hljs.highlight(event.target.value, {
                  language: "cypher",
                }).value;
              }
            }}
            placeholder="Please input query"
          />
          <pre
            style={{
              position: "absolute",
              top: 2,
              left: 1,
              width: 400,
              textAlign: "left",
              padding: "4px 11px",
              verticalAlign: "bottom",
            }}
          >
            <div ref={markup}></div>
          </pre>
        </div>
      ) : (
        <></>
      )}
      <Handle type="target" position={Position.Top} id="t" />
      <Handle type="source" position={Position.Bottom} id="b" />
      <Handle type="target" position={Position.Left} id="l" />
      <Handle type="source" position={Position.Right} id="r" />
    </div>
  );
}

export interface SinkNodeProps {
  data: { label: ReactNode; width: number; upstream: string };
}
function SinkNode({ data }: SinkNodeProps) {
  return (
    <div
      style={{
        border: "2px solid #535bf2",
        background: "#535bf2",
        borderRadius: 5,
        color: "white",
        width: data.width,
        textAlign: "center",
      }}
    >
      <div style={{ padding: "4px 15px" }}>{data.label}</div>
      <Handle type="target" position={Position.Left} id="l" />
    </div>
  );
}

const nodeTypes = {
  source: SourceNode,
  procedure: ProcedureNode,
  sink: SinkNode,
};
export default nodeTypes;
