import { useEffect } from "react";
import "./App.css";
import { wasm_init } from "c-wasm-interpreter";

let init = false;

function App() {
    useEffect(() => {
        if (!init) {
            init = true;
            wasm_init();
        }
    });

    return (
        <div>Hello</div>
    );
};

export default App;
