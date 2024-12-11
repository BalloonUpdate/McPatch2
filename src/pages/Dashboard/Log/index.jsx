import React, {useEffect, useRef, useState} from "react";
import {Button, Input, message, Modal, Popconfirm, Select} from "antd";
import {
  taskCombineRequest, taskPackRequest,
  taskRevertRequest,
  taskTestRequest,
  taskUploadRequest,
  taskStatusRequest
} from "@/api/task.js";
import {terminalFullRequest, terminalMoreRequest} from "@/api/terminal.js";
import {RotateCcw} from "lucide-react";
import {generateRandomStr} from "@/utils/tool.js";

const {TextArea} = Input;

const options = [
  {value: 5000, label: '5s'},
  {value: 1000, label: '1s'},
  {value: 15000, label: '15s'}
]

const Index = () => {

  const [logs, setLogs] = useState([])
  const [packShow, setPackShow] = useState(false)
  const [version, setVersion] = useState('');
  const [updateRecord, setUpdateRecord] = useState('');
  const [refreshInterval, setRefreshInterval] = useState(options[0].value);
  const logsRef = useRef(null);
  const [messageApi, contextHolder] = message.useMessage();

  useEffect(() => {
    terminalFull()
  }, []);

  useEffect(() => {
    if (logsRef.current) {
      logsRef.current.scrollTop = logsRef.current.scrollHeight;
    }
  }, [logs]);

  useEffect(() => {
    const intervalId = setInterval(() => {
      terminalMore()
    }, refreshInterval)

    return () => clearInterval(intervalId);
  }, [refreshInterval])

  const terminalFull = async () => {
    const {code, msg, data} = await terminalFullRequest();
    if (code === 1) {
      setLogs(data.content)
    }
  }

  const terminalMore = async () => {
    const {code, msg, data} = await terminalMoreRequest();
    if (code === 1) {
      if (data.content.length === 0) {
        return
      }
      setLogs(prev => [...prev, ...data.content])
    }
  }

  const changeRefreshInterval = (value) => {
    setRefreshInterval(value)
  };

  const taskPack = async () => {
    const tempVersion = version === '' ? generateRandomStr() : version
    const tempUpdateRecord = updateRecord === '' ? '这个人很懒, 没有写更新记录.' : updateRecord

    const {code, msg, data} = await taskPackRequest(tempVersion, tempUpdateRecord);
    if (code === 1) {
      messageApi.success('打包成功.')
    } else {
      messageApi.error(msg)
    }
    setPackShow(false)
  }

  const taskCombine = async () => {
    const {code, msg, data} = await taskCombineRequest();
    if (code === 1) {
      messageApi.success('合并成功.')
    } else {
      messageApi.error(msg)
    }
  }

  const taskTest = async () => {
    const {code, msg, data} = await taskTestRequest();
    if (code === 1) {
      messageApi.success('测试成功.')
    } else {
      messageApi.error(msg)
    }
  }

  const taskRevert = async () => {
    const {code, msg, data} = await taskRevertRequest();
    if (code === 1) {
      messageApi.success('回退成功.')
    } else {
      messageApi.error(msg)
    }
  }

  const taskUpload = async () => {
    const {code, msg, data} = await taskUploadRequest();
    if (code === 1) {
      messageApi.success('任务已提交.')
    } else {
      messageApi.error(msg)
    }
  }

  const taskStatus = async () => {
    const {code, msg, data} = await taskStatusRequest();
    if (code === 1) {
      messageApi.success('任务已提交.')
    } else {
      messageApi.error(msg)
    }
  }

  const copy = async (item) => {
    await navigator.clipboard.writeText(`${showTime(item.time)}-${item.level}-${item.content}`);
    messageApi.success('复制成功!')
  }

  const getTextColor = (level) => {
    if (level === 'debug') return 'text-zinc-500';
    if (level === 'info') return 'text-white';
    if (level === 'warning') return 'text-yellow-600';
    if (level === 'error') return 'text-[#FF0000]';
  };

  const showTime = (timestamp) => {
    return new Date(timestamp * 1000).toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      hour12: false
    });
  }

  return (
    <>
      {contextHolder}
      <div className="flex flex-col p-10 min-h-screen">
        <div className="flex justify-start items-center h-8">
          <Popconfirm title="风险操作,请再次确认!" onConfirm={taskStatus} okText="确定" cancelText="取消">
            <Button type="primary" size="large">检查文件修改</Button>
          </Popconfirm>
          <Popconfirm title="风险操作,请再次确认!" onConfirm={taskTest} okText="确定" cancelText="取消">
            <Button type="primary" size="large" className="ml-2">测试更新包</Button>
          </Popconfirm>
          <Popconfirm title="风险操作,请再次确认!" onConfirm={taskUpload} okText="确定" cancelText="取消">
            <Button type="primary" size="large" className="ml-2">上传public目录</Button>
          </Popconfirm>
          <Button type="primary" size="large" className="ml-2" onClick={() => setPackShow(true)}>打包新版本</Button>
          <Popconfirm title="风险操作,请再次确认!" onConfirm={taskRevert} okText="确定" cancelText="取消">
            <Button type="primary" size="large" className="ml-2">回退整个工作空间</Button>
          </Popconfirm>
          <Popconfirm title="风险操作,请再次确认!" onConfirm={taskCombine} okText="确定" cancelText="取消">
            <Button type="primary" size="large" className="ml-2">合并更新包</Button>
          </Popconfirm>
          <Select
            defaultValue={options[0].value}
            size={"large"}
            className="ml-auto w-40"
            onChange={changeRefreshInterval}
            options={options}/>
          <Button type="primary" size="large" className="ml-2" icon={<RotateCcw size={20} strokeWidth={1.5}/>}
                  onClick={terminalMore}/>

        </div>
        <div
          ref={logsRef}
          className="flex-1 mt-8 bg-black dark:bg-gray-800 text-white overflow-auto min-h-[calc(100vh-160px)] max-h-[calc(100vh-160px)]">
          {
            logs.map((item, index) => {
              return (
                <div
                  key={index}
                  onClick={() => copy(item)}
                  className="flex items-center pt-0.5 pb-0.5 pl-2 text-base text-gray-300 rounded cursor-pointer select-none hover:bg-gray-700 duration-200">
                  <span className="w-48">[{showTime(item.time)}]</span>
                  {/*<span className={`w-24 ${getTextColor(item.level)}`}>[{item.level}]</span>*/}
                  <span className={`${getTextColor(item.level)}`}>{item.content}</span>
                </div>
              )
            })
          }
        </div>
      </div>
      <Modal
        title="打包"
        okText="确认"
        cancelText="取消"
        open={packShow}
        onOk={taskPack}
        onCancel={() => setPackShow(false)}>
        <div>
          <div className="text-base text-gray-400">版本号与详情均可不填,使用默认参数.</div>
          <Input
            className="mt-5"
            placeholder="请输入版本号."
            value={version}
            onChange={(e) => setVersion(e.target.value)}/>
          <TextArea
            className="mt-2 mb-5"
            placeholder="请输入更新记录."
            autoSize={{maxRows: 10, minRows: 4}}
            maxLength={4000}
            value={updateRecord}
            onChange={(e) => setUpdateRecord(e.target.value)}/>
        </div>
      </Modal>
    </>
  );
};

export default Index;
