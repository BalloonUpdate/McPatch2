import {fsDiskInfoRequest} from "@/api/fs.js";
import {useEffect, useState} from "react";

const Index = () => {
  const [diskInfo, setDiskInfo] = useState({total: 0, used: 0});

  const getDiskInfo = async () => {
    const {code, msg, data} = await fsDiskInfoRequest();
    if (code === 1) {
      setDiskInfo(data)
    }
  }

  useEffect(() => {
    getDiskInfo()
  }, []);

  return (
    <>
      <div className="p-10 min-h-screen">
        <div className="text-2xl font-bold text-indigo-600">磁盘使用量</div>
        <div className='h-8 mt-2 mb-2 bg-indigo-100 dark:bg-dark-3 relative w-full rounded-2xl'>
          <div
            className='bg-indigo-600 absolute top-0 left-0 flex h-full items-center justify-center rounded-2xl text-xs font-semibold text-white'
            style={{width: `${(diskInfo.used / diskInfo.total * 100).toFixed(2)}%`}}>
          </div>
        </div>
        <div className="text-base flex justify-end items-center w-full">
          <div>{(diskInfo.used / 1024 / 1024 / 1024).toFixed(2)}GB / {(diskInfo.total / 1024 / 1024 / 1024).toFixed(2)}GB</div>
          <div className="ml-2">{(diskInfo.used / diskInfo.total * 100).toFixed(2)}%</div>
        </div>
      </div>
    </>
  );
};

export default Index;
