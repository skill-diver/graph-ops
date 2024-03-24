import SaveOutlined from "@ant-design/icons/SaveOutlined";
import { Switch } from "antd";
import Button from "antd/es/button";
import Menu from "antd/es/menu";
import React, {
  DragEvent,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";
import ReactFlow, {
  Background,
  BackgroundVariant,
  Controls,
  MarkerType,
  MiniMap,
  NodeRemoveChange,
  ReactFlowInstance,
  ReactFlowProvider,
  addEdge,
  useEdgesState,
  useNodesState,
} from "reactflow";
import "reactflow/dist/style.css";

import AddInput from "./AddInput";
import ConfigModal, { FormState } from "./ConfigModal";
import nodeTypes, { ProcedureList, SinkNodeProps } from "./Nodes";
import WorkFlow from "./Workflow";

function capitalizeWords(str: string, split: string) {
  return str
    .toLowerCase()
    .split(split)
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");
}

const SubmitButton = (props: {
  handleSubmit: (callback: () => void) => void;
}) => {
  const [loading, setLoading] = useState(false);
  return (
    <Button
      loading={loading}
      onClick={() => {
        setLoading(true);
        props.handleSubmit(() => {
          setLoading(false);
        });
      }}
      type="ghost"
      icon={<SaveOutlined />}
    >
      Save
    </Button>
  );
};

enum Actions {
  None = "none",
  Add = "add",
}

type WorkflowEditorProps = {
  workflowId?: string;
  onFinish: (workflowName: string) => void;
};
export default function WorkflowEditor({
  workflowId,
  onFinish,
}: WorkflowEditorProps) {
  const [action, setAction] = useState<{ op: Actions; data?: string }>({
    op: Actions.None,
  });
  const [featureLogicMode, setFeatureLogicMode] = useState(true);

  /* data */
  const [inputList, setInputList] = useState<Map<string, boolean>>(new Map());
  const [graphProcedureList, setGraphProcedureList] = useState<ProcedureList>(
    []
  );
  const [dfProcedureList, setDfProcedureList] = useState<ProcedureList>([]);

  const fetchGraphProcedureList = async () => {
    const url = import.meta.env.VITE_OFNIL_BACKEND_URL + "/gaf";
    const response: Response = await fetch(url);
    if (!response.ok) throw new Error(response.status.toString());
    const data = await response.json();
    setGraphProcedureList(
      data.map((record: string) => ({
        key: record,
        label: capitalizeWords(record, "_"),
        outputNodeType: "procedure",
      }))
    );
  };
  useEffect(() => {
    fetchGraphProcedureList();
    // TODO(tatiana): hardcode now
    setDfProcedureList([
      { key: "select", label: "Select", outputNodeType: "procedure" },
      { key: "aggregation", label: "Aggregation", outputNodeType: "procedure" },
      { key: "filter", label: "Filter", outputNodeType: "procedure" },
      // { key: "sql", label: "SQL", outputNodeType: "procedure" },
    ]);
  }, []);

  /* reactflow */
  const reactFlowWrapper = useRef<HTMLDivElement>(null);
  const [nodes, setNodes, onNodesChange] = useNodesState([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);
  const [reactFlowInstance, setReactFlowInstance] =
    useState<ReactFlowInstance>();

  const addInput = (
    resource_id: string,
    position: { x: number; y: number }
  ) => {
    setNodes((nds) =>
      nds.concat({
        id: resource_id,
        data: {
          label: resource_id,
          procedureList: graphProcedureList,
        },
        position,
        type: "source",
      })
    );
    setInputList(inputList.set(resource_id, true));
  };

  const removeInput = (resource_id: string) => {
    setInputList(inputList.set(resource_id, false));
  };

  const onDragOver = useCallback((event: DragEvent) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = "move";
  }, []);
  const onDrop = useCallback(
    (event: DragEvent) => {
      event.preventDefault();
      if (!reactFlowWrapper.current || !reactFlowInstance) return;
      const nodeType = event.dataTransfer.getData("app/reactflow");
      // check if the dropped element is valid
      if (typeof nodeType === "undefined" || !nodeType) return;

      const reactFlowBounds = reactFlowWrapper.current.getBoundingClientRect();

      const itemX = parseFloat(event.dataTransfer.getData("app/reactflow/x"));
      const itemY = parseFloat(event.dataTransfer.getData("app/reactflow/y"));
      const position = reactFlowInstance.project({
        x: event.clientX - reactFlowBounds.left - itemX,
        y: event.clientY - reactFlowBounds.top - itemY + 1, // plus 1 because of border width
      });

      const key = event.dataTransfer.getData("app/reactflow/key");
      const upstream = event.dataTransfer.getData("app/reactflow/upstream");
      const id = upstream + "/" + key;
      let newNode = true;
      const label = capitalizeWords(key, "_");
      setNodes((nds) => {
        // TODO(tatiana): unselect upstream onDragStart
        nds.forEach((node) => {
          if (node.id == upstream) {
            node.selected = false;
          }
        });
        // check existing node
        // FIXME(tatiana): allow multiple (e.g. pagerank) nodes with different ids
        const existing_node = nds.find((node) => {
          node.id == id;
        });
        if (existing_node) {
          newNode = false;
          existing_node.position = position;
          return nds;
        }
        return nds.concat({
          id,
          type: nodeType,
          position,
          data: {
            isQuery:
              key == "cypher" ||
              key == "select" ||
              key == "aggregate" ||
              key == "filter",
            width: parseFloat(
              event.dataTransfer.getData("app/reactflow/width")
            ),
            label,
            upstream, // applies only to procedure node
            // TODO(tatiana): distinguish graph and df for nodeType procedure and set appropriate list. hardcode df now
            // applies only to procedure node
            procedureList: dfProcedureList.concat({
              key: "export",
              label: "Export",
              outputNodeType: "sink",
            }),
          },
        });
      });

      if (newNode) {
        setEdges((edges) =>
          addEdge(
            {
              source: upstream,
              target: id,
              sourceHandle: "r",
              targetHandle: "l",
              animated: true,
            },
            edges
          )
        );
      }

      if (!featureLogicMode) {
        setConfigModalTarget(id);
        setConfigTitle(label);
      }
    },
    [reactFlowInstance, featureLogicMode]
  );

  /* config modal*/
  const [configModalTarget, setConfigModalTarget] = useState<string>("");
  const [configTitle, setConfigTitle] = useState<string>("");
  const [configs, setConfigs] = useState<Map<string, FormState>>(new Map());
  const onConfigSave = (key: string, value: FormState) => {
    setConfigs((configs) => configs.set(key, value));
    setConfigModalTarget("");
  };

  /* restore */
  const restoreEditor = async () => {
    const restoreEditorFromObj = (str: string) => {
      const editorState = JSON.parse(str);
      if (editorState.flow) {
        const { x = 0, y = 0, zoom = 1 } = editorState.flow.viewport;
        setNodes(editorState.flow.nodes || []);
        setEdges(editorState.flow.edges || []);
        reactFlowInstance?.setViewport({ x, y, zoom });
      }
      const confs = new Map<string, FormState>(
        Object.keys(editorState.configs).map((key) => {
          return [key, { values: editorState.configs[key], configs: null }];
        })
      );
      setConfigs(confs);
    };
    fetch(
      import.meta.env.VITE_OFNIL_BACKEND_URL +
        "/transformation?id=" +
        workflowId
    ).then((response) => {
      if (response.ok) {
        response.json().then((workflow: WorkFlow) => {
          restoreEditorFromObj(workflow.body);
        });
      } else {
        console.warn("error", response);
      }
    });
  };

  useEffect(() => {
    if (workflowId) restoreEditor();
  }, [workflowId]);

  return (
    <>
      <ReactFlowProvider>
        <Menu
          style={{ marginBottom: "10px", backgroundColor: "#f5f5f5" }}
          mode="horizontal"
          id="workflow-edit-controls"
          items={[
            {
              key: "input",
              label: (
                <AddInput
                  onClick={(resource_id) => {
                    setAction({ op: Actions.Add, data: resource_id });
                  }}
                  inputList={inputList}
                  onInputListUpdate={(newInputList) =>
                    setInputList(newInputList)
                  }
                />
              ),
            },
            {
              key: "submit",
              label: (
                <SubmitButton
                  handleSubmit={(callback) => {
                    if (reactFlowInstance) {
                      let workflowName: string;
                      if (workflowId) {
                        const splits = workflowId.split("/");
                        workflowName = splits[splits.length - 1];
                      } else {
                        workflowName = "WORKFLOW" + Date.now().toString();
                      }

                      const workflow = createWorkFlow(
                        workflowName,
                        reactFlowInstance,
                        configs
                      );
                      submitWorkflow(workflow, callback);
                      onFinish(workflowName);
                    }
                  }}
                />
              ),
            },
            {
              key: "featureLogicMode",
              label: (
                <Switch
                  checkedChildren={"Feature Logic Mode"}
                  unCheckedChildren={"Workflow Mode"}
                  defaultChecked={featureLogicMode}
                  onChange={(mode) => {
                    setFeatureLogicMode(mode);
                  }}
                />
              ),
            },
          ]}
        ></Menu>
        <div
          style={{ width: "100%", height: "calc(100% - 56px)" }}
          ref={reactFlowWrapper}
        >
          <ReactFlow
            id="workflow-editor"
            nodes={nodes}
            edges={edges}
            nodeTypes={nodeTypes}
            defaultEdgeOptions={{
              style: { strokeWidth: 1, stroke: "black" },
              animated: true,
              markerEnd: {
                type: MarkerType.ArrowClosed,
                color: "black",
              },
            }}
            className={`workflow-editor-${action.op}`}
            onInit={setReactFlowInstance}
            selectionOnDrag={true}
            panOnDrag={[1, 2]}
            /* TODO(tatiana): refactor with drag-and-drop ...*/
            onNodesChange={(changes) => {
              changes.forEach((change) => {
                if (change.type == "remove") {
                  removeInput((change as NodeRemoveChange).id);
                }
              });
              onNodesChange(changes);
            }}
            onClick={(e) => {
              if (action.op == Actions.Add && action.data) {
                e.preventDefault();
                const target = e.currentTarget.getBoundingClientRect();
                addInput(action.data, {
                  x: e.clientX - target.x,
                  y: e.clientY - target.y,
                });
                setAction({ op: Actions.None });
              }
            }}
            /* ... TODO(tatiana): refactor with drag-and-drop*/
            onNodeDoubleClick={(event, node) => {
              if (node.type != "source") {
                setConfigModalTarget(node.id);
                setConfigTitle(node.data.label);
              }
            }}
            onEdgesChange={onEdgesChange}
            onDragOver={onDragOver}
            onDrop={onDrop}
          >
            <Controls />
            <MiniMap position="bottom-right" />
            <Background variant={BackgroundVariant.Dots} gap={12} size={1} />
          </ReactFlow>
        </div>
        <ConfigModal
          title={configTitle}
          open={configModalTarget.length > 0}
          target={configModalTarget}
          onSave={onConfigSave}
          formState={configs.get(configModalTarget)}
          onCancel={() => {
            setConfigModalTarget("");
          }}
        />
      </ReactFlowProvider>
    </>
  );
}

const submitWorkflow = (workflow: WorkFlow, callback: () => void) => {
  const post_url = import.meta.env.VITE_OFNIL_BACKEND_URL + "/transformation";
  const workflowStr = JSON.stringify(workflow);
  fetch(post_url, {
    method: "POST",
    body: workflowStr,
  }).then((response) => {
    if (response.status == 200) {
      callback();
    } else {
      console.warn("error", response);
    }
  });
};

const createWorkFlow = (
  name: string,
  reactFlowInstance: ReactFlowInstance,
  configs: Map<string, FormState>
) => {
  const opConfigs: { [x: string]: object } = {};
  configs.forEach((value, key) => {
    if (value?.values) opConfigs[key] = value.values;
  });
  const workflow: WorkFlow = {
    export_resources: [],
    source_field_ids: [],
    owners: ["Ofnil"],
    body: JSON.stringify({
      configs: opConfigs,
      flow: reactFlowInstance.toObject(),
    }),
    description: "",
    name,
    variant: {
      Default: [],
    },
    tags: new Map(),
  };

  // get export resources
  // FIXME(tatiana): hardcode now
  reactFlowInstance.getNodes().forEach((node, node_index) => {
    if (node.type == "sink") {
      const upstream_config = configs.get(
        (node as SinkNodeProps).data.upstream
      )?.values;
      if (upstream_config) {
        const feature_names = upstream_config[
          "output_feature.feature_name(s)"
        ] as string[];
        const entity_label = upstream_config[
          "output_feature.target_vertex"
        ] as string;
        workflow.export_resources = workflow.export_resources.concat(
          feature_names.map((name) => {
            return [
              node_index,
              "default/Field/" + entity_label.toLowerCase() + "/" + name,
            ];
          })
        );
      }
    }
  });
  return workflow;
};
