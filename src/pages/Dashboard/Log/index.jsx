import React, {useEffect, useRef, useState} from "react";
import {Button, Input, message, Modal} from "antd";
import {taskCombineRequest, taskPackRequest, taskRevertRequest, taskTestRequest} from "@/api/task.js";
import {terminalFullRequest, terminalMoreRequest} from "@/api/terminal.js";
import {RotateCcw} from "lucide-react";
import {generateRandomStr} from "@/utils/tool.js";

const Index = () => {

  const [logs, setLogs] = useState([])
  const [packShow, setPackShow] = useState(false)
  const [version, setVersion] = useState('');
  const [updateRecord, setUpdateRecord] = useState('');
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
        messageApi.warning('未产生任何新日志.')
      }
      setLogs(prev => [...prev, ...data.content])
    }
  }

  const taskPack = async () => {
    const tempVersion = version === '' ? generateRandomStr() : version
    const tempUpdateRecord = updateRecord === '' ? '这个人很懒, 没有写更新记录.' : updateRecord

    const {code, msg, data} = await taskPackRequest(tempVersion, tempUpdateRecord);
    if (code === 1) {
      messageApi.success('打包成功.')
      await terminalMore()
    } else {
      messageApi.error(msg)
    }
    setPackShow(false)
  }

  const taskCombine = async () => {
    const {code, msg, data} = await taskCombineRequest();
    if (code === 1) {
      messageApi.success('合并成功.')
      await terminalMore()
    } else {
      messageApi.error(msg)
    }
  }

  const taskTest = async () => {
    const {code, msg, data} = await taskTestRequest();
    if (code === 1) {
      messageApi.success('测试成功.')
      await terminalMore()
    } else {
      messageApi.error(msg)
    }
  }

  const taskRevert = async () => {
    const {code, msg, data} = await taskRevertRequest();
    if (code === 1) {
      messageApi.success('回退成功.')
      await terminalMore()
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
      <div className="flex flex-col min-h-[calc(100vh-80px)]">
        <div className="flex justify-start items-center h-8">
          <Button type="primary" size="large" icon={<RotateCcw size={20} strokeWidth={1.5}/>} onClick={terminalMore}/>
          <Button type="primary" size="large" className="ml-2" onClick={() => setPackShow(true)}>打包</Button>
          <Button type="primary" size="large" className="ml-2" onClick={taskCombine}>合并</Button>
          <Button type="primary" size="large" className="ml-2" onClick={taskTest}>测试</Button>
          <Button type="primary" size="large" className="ml-2" onClick={taskRevert}>回退</Button>
        </div>
        <div
          ref={logsRef}
          className="flex-1 mt-4 bg-black text-white overflow-auto min-h-[calc(100vh-160px)] max-h-[calc(100vh-160px)]">
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
          <Input
            className="mt-2 mb-5"
            placeholder="请输入更新记录."
            value={updateRecord}
            onChange={(e) => setUpdateRecord(e.target.value)}/>
        </div>
      </Modal>
    </>
  );
};

export default Index;
