import React, {useState} from 'react';
import {Button, Input, message, Modal, Upload} from "antd";
import {fsUploadRequest, fsMakeDirectoryRequest} from "@/api/fs.js";

const Index = ({path, getFileList}) => {

  const [makeDirectoryShow, setMakeDirectoryShow] = useState(false)
  const [directory, setDirectory] = useState('');
  const [uploadFile, setUploadFile] = useState(null)
  const [uploadFileList, setUploadFileList] = useState([]);
  const [messageApi, contextHolder] = message.useMessage();

  const makeDirectory = async () => {
    let key = path.join('/');
    key = key.length === 0 ? directory : `${key}/${directory}`
    const {code, msg, data} = await fsMakeDirectoryRequest(key);
    if (code === 1) {
      setMakeDirectoryShow(false)
      messageApi.success('文件夹创建成功.')
      getFileList()
    } else {
      messageApi.error(msg)
    }
  }

  const uploadFileProps = {
    showUploadList: false,
    multiple: false,
    maxCount: 1,
    customRequest: async (options) => {
      const {file, onSuccess, onError, onProgress} = options;
      let key = path.join('/');
      key = key.length === 0 ? file.name : `${key}/${file.name}`
      
      try {
        const res = await fsUploadRequest(key, file, onProgress);
        onSuccess(res);
      } catch (error) {
        onError(error);
      }
    },
    onChange: (info) => {
      if (info.file.status === 'done') {
        messageApi.success('上传成功.')
        getFileList()
      } else if (info.file.status === 'error') {
        messageApi.error('上传失败.')
      }
    }
  };

  const uploadFileExplorerProps = {
    showUploadList: false,
    directory: true,
    multiple: false,
    maxCount: 1,
    customRequest: async (options) => {
      const {file, onSuccess, onError, onProgress} = options;
      let key = path.join('/');
      key = key.length === 0 ? file.webkitRelativePath : `${key}/${file.webkitRelativePath}`

      try {
        const res = await fsUploadRequest(key, file, onProgress);
        onSuccess(res);
      } catch (error) {
        onError(error);
      }
    },
    onChange: (info) => {
      if (info.file.status === 'done') {
        messageApi.success('上传成功.')
        getFileList()
      } else if (info.file.status === 'error') {
        messageApi.error('上传失败.')
      }
    }
  };

  return (
    <>
      {contextHolder}
      <div className="flex justify-start items-center h-8">
        <Button type="primary" size="large" onClick={() => setMakeDirectoryShow(true)}>创建文件夹</Button>
        <Upload
          {...uploadFileProps}
          className="ml-2">
          <Button type="primary" size="large">上传文件</Button>
        </Upload>
        <Upload
          {...uploadFileExplorerProps}
          className="ml-2">
          <Button type="primary" size="large">上传文件夹</Button>
        </Upload>
      </div>

      <Modal
        title="创建文件夹"
        okText="确认"
        cancelText="取消"
        open={makeDirectoryShow}
        onOk={() => makeDirectory()}
        onCancel={() => setMakeDirectoryShow(false)}>
        <div>
          <Input
            className="mt-2"
            placeholder="请输入文件夹名称."
            value={directory}
            onChange={(e) => setDirectory(e.target.value)}/>
        </div>
      </Modal>
    </>
  );
};

export default Index;
