import { memo } from "react";
import { type Clip } from "./types";
import CustomWavesurferPlayer from "./CustomWavesurferPlayer";

const VarPlayer = memo(function VarPlayer({
  // false positive
  // eslint-disable-next-line react/prop-types
  clips,
}: {
  clips: { [name: string]: Clip };
}) {
  return Object.entries(clips).map(([name, clip]) => (
    <>
      <p>{name}:</p>
      <CustomWavesurferPlayer key={name} clip={clip}></CustomWavesurferPlayer>
    </>
  ));
});
export default VarPlayer;
