import React, {useEffect, useState} from 'react';
import FileBreadcrumb from "@/components/FileBreadcrumb/index.jsx";
import {fsListRequest} from "@/api/fs.js";
import TileViewFileExplorer from "@/components/TileViewFileExplorer/index.jsx";
import FolderButtonGroup from "@/components/FolderButtonGroup/index.jsx";

const Index = () => {
  const [path, setPath] = useState([]);
  const [fileList, setFileList] = useState([])

  const getFileList = async () => {
    const {code, msg, data} = await fsListRequest(path.join('/'));
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
        <div className="mt-2 pt-4 pb-2">
          <FolderButtonGroup path={path} getFileList={getFileList}/>
        </div>
        <div className="flex-1 mt-2 h-full bg-gray-100">
          <TileViewFileExplorer
            path={path}
            getFileList={getFileList}
            items={fileList}
            handlerNextPath={handlerNextPath}/>
        </div>
      </div>
    </>
  );
};

export default Index;
