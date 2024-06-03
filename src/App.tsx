import {useState} from "react"

import taurpc from "./proxy";

export default function App() {
	const [code, setCode] = useState("");
	return <>
		<textarea value={code} onChange={e => setCode(e.target.value)} />
		<button onClick={() => {
			taurpc.run_code(code);
			taurpc.audio.play_from_stack()
		}}>Hoohoo</button>
	</>
}