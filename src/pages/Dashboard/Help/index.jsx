import {SquareArrowOutUpRight} from "lucide-react";

const Index = () => {
  return (
    <>
      <div className="flex items-center mt-2 text-base font-bold text-indigo-600">
        <span>官方文档:</span>
        <a className="flex items-center ml-2 text-gray-500 w-40" target="_blank"
           href="https://balloonupdate.github.io/McPatchDocs/">
          [打开<SquareArrowOutUpRight size={12} strokeWidth={1.5}/>]
        </a>
      </div>
    </>
  );
};

export default Index;
