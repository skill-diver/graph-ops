import PlusOutlined from "@ant-design/icons/PlusOutlined";
import Button from "antd/es/button";
import Dropdown from "antd/es/dropdown";
import type { MenuProps } from "antd/es/menu";
import React, { useState } from "react";

type AddInputProps = {
  onClick: (resource_id: string) => void;
  inputList: Map<string, boolean>; // resource id: disabled
  onInputListUpdate: (newInputList: Map<string, true>) => void;
};
export default function AddInput({
  onClick,
  inputList,
  onInputListUpdate,
}: AddInputProps) {
  const [loading, setLoading] = useState(false);

  const initDropdown = (
    e:
      | React.MouseEvent<HTMLAnchorElement, MouseEvent>
      | React.MouseEvent<HTMLButtonElement, MouseEvent>
  ) => {
    e.preventDefault();
    if (inputList.size == 0) {
      setLoading(true);
      const url = import.meta.env.VITE_OFNIL_BACKEND_URL + "/graphs";
      fetch(url)
        .then((response) => {
          if (!response.ok) throw new Error(response.status.toString());
          response.json().then((data) => {
            const newInputList = new Map();
            Object.keys(data).forEach((id) => newInputList.set(id, false));
            onInputListUpdate(newInputList);
            setLoading(false);
          });
        })
        .catch((e) => console.log(e));
    }
  };

  const menu: MenuProps["items"] = [];
  inputList.forEach((value, key) => {
    menu.push({
      label: key,
      key,
      disabled: value,
    });
  });

  return (
    <Dropdown
      autoFocus={true}
      trigger={["click"]}
      menu={{
        items: menu,
        onClick: (info) => onClick(info.key),
      }}
      destroyPopupOnHide={true}
    >
      <Button
        icon={<PlusOutlined />}
        type="ghost"
        loading={loading}
        key="add_input"
        onClick={(event) => initDropdown(event)}
      >
        Add Input
      </Button>
    </Dropdown>
  );
}
