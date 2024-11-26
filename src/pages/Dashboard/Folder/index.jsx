import React, {useEffect, useState} from 'react';
import FileBreadcrumb from "@/components/FileBreadcrumb/index.jsx";
import {getFileListRequest} from "@/api/fs.js";
import TileViewFileExplorer from "@/components/TileViewFileExplorer/index.jsx";

const Index = () => {
  const [path, setPath] = useState([]);
  const [fileList, setFileList] = useState([])

  const getFileList = async () => {
    const {code, msg, data} = await getFileListRequest(path.join('/'));
    if (code === 1) {
      setFileList(data.files)
    }
  }

  const handlerNextPath = (item) => {
    setPath(prev => [...prev, item.name]);
  }

  const handlerBreadcrumb = (index) => {
    setPath(prev => prev.slice(0, index));
  }

  useEffect(() => {
    getFileList()
  }, [path]);

  return (
    <>
      <div className="flex flex-col min-h-[calc(100vh-5rem)]">
        <div>
          <FileBreadcrumb path={path} handlerBreadcrumb={handlerBreadcrumb}/>
        </div>
        <div className="flex-1 mt-2 h-full bg-gray-50">
          <TileViewFileExplorer items={fileList} handlerNextPath={handlerNextPath}/>
        </div>
      </div>
    </>
  );
};

export default Index;
