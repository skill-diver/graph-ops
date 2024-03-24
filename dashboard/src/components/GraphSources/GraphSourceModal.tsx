import { PlusOutlined } from "@ant-design/icons";
import { Button, Modal } from "antd";
import React, { useState } from "react";

import FirstForm from "./Forms/FirstForm";
import NextForm from "./Forms/NextForm";

export interface GraphSourceMeta {
  data_source_type: string | undefined;
  data_source_name: string | undefined;
  database: string | undefined;
  name: string | undefined;
  variant: string | undefined;
}

export default function RegisterGraphSourceModal(props: {
  onFinish: () => void;
}) {
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [index, setIndex] = useState(0);
  const [sourceMeta, setSourceMeta] = useState<GraphSourceMeta>({
    data_source_type: undefined,
    data_source_name: undefined,
    database: undefined,
    name: undefined,
    variant: undefined,
  });

  const showModal = () => {
    setIsModalOpen(true);
  };
  const handleOk = () => {
    setIsModalOpen(false);
  };
  const handleCancel = () => {
    setIsModalOpen(false);
  };

  return (
    <>
      <Button type="primary" icon={<PlusOutlined />} onClick={showModal}>
        New Graph Source
      </Button>
      <Modal
        title="Register New Graph Source"
        open={isModalOpen}
        onOk={handleOk}
        onCancel={handleCancel}
        footer={null}
      >
        {
          [
            <FirstForm
              key={index}
              setIndex={setIndex}
              setSourceMeta={setSourceMeta}
            />,
            <NextForm
              key={index}
              setIndex={setIndex}
              setIsModalOpen={setIsModalOpen}
              onFinish={props.onFinish}
              sourceMeta={sourceMeta}
            />,
          ][index]
        }
      </Modal>
    </>
  );
}
