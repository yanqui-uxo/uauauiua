import {useState} from "react"

import Wavesurfer from "wavesurfer.js";
import WavesurferPlayer from "@wavesurfer/react";

import taurpc from "./proxy";

export default function App() {
	const [code, setCode] = useState('P ~ "../prelude.ua"\n');
	const [wavesurfer, setWavesurfer] = useState<Wavesurfer | null>(null);
	const [error, setError] = useState("");

	return <>
		<textarea value={code} onChange={e => setCode(e.target.value)} />
		{wavesurfer && <button onClick={async () => {
			try {
				const newCode = await taurpc.format_code(code);
				setCode(newCode);
				const data = await taurpc.run_code(newCode);
				console.log(data);
				wavesurfer.load(URL.createObjectURL(new Blob([new Uint8Array(data.stackWav)])));
			} catch (e) {
				setError(`Error: ${e} (time: ${Date.now()})`);
			}
		}}>Run code</button>}
		<WavesurferPlayer onInit={ws => setWavesurfer(ws)} onInteraction={ws => ws.play()} onFinish={ws => ws.setTime(0)} />
		<p>{error}</p>
	</>
}