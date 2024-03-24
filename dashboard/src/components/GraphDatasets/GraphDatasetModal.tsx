import { PlusOutlined } from "@ant-design/icons";
import { Button, Modal } from "antd";
import React, { useState } from "react";

import GraphDatasetForm from "./GraphDatasetForm";

export default function RegisterGraphDatasetModal(props: {
  onFinish: () => void;
}) {
  const [isModalOpen, setIsModalOpen] = useState(false);

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
        New Graph Dataset
      </Button>
      <Modal
        title="Register New Graph Dataset"
        open={isModalOpen}
        onOk={handleOk}
        onCancel={handleCancel}
        footer={null}
      >
        <GraphDatasetForm
          onFinish={() => {
            props.onFinish();
            setIsModalOpen(false);
          }}
        />
      </Modal>
    </>
  );
}
