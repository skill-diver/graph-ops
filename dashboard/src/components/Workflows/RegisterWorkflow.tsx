import PlusOutlined from "@ant-design/icons/PlusOutlined";
import { message } from "antd";
import Button from "antd/es/button";
import Drawer from "antd/es/drawer";
import React, { useState } from "react";

import WorkflowEditor from "./WorkflowEditor";

export default function RegisterWorkflow(props: { onFinish: () => void }) {
  const [open, setOpen] = useState(false);
  return (
    <>
      <Button
        type="primary"
        icon={<PlusOutlined />}
        onClick={() => setOpen(true)}
      >
        New Workflow
      </Button>
      <Drawer
        className="workflow-drawer"
        width={"calc( 100% - 80px )"}
        // maskClosable={false}
        open={open}
        title={
          <div className="flex">
            <span>Create a New Workflow</span>
          </div>
        }
        onClose={() => setOpen(false)}
      >
        <WorkflowEditor
          onFinish={(workflowName) => {
            message.info("Created workflow " + workflowName);
            setOpen(false);
            props.onFinish();
          }}
        />
        ;
      </Drawer>
    </>
  );
}
