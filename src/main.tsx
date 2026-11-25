import { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { getInstallationInfo, type InstallationInfo } from "./lib/desktop";
import icon from "./assets/picaso.svg";
import "./styles.css";

function App() {
  const [info, setInfo] = useState<InstallationInfo>();
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    getInstallationInfo().then(setInfo).catch(() => setFailed(true));
  }, []);

  return (
    <main>
      <header className="flex items-center gap-4">
        <img src={icon} alt="" width="56" height="56" />
        <h1>Picaso</h1>
      </header>
      {info ? (
        <dl>
          <div><dt>Version</dt><dd>{info.version}</dd></div>
          <div><dt>Data folder</dt><dd className="path">{info.dataDirectory}</dd></div>
        </dl>
      ) : (
        <p role="status">
          {failed ? "Couldn’t open the local data folder. Check your account’s access and reopen Picaso." : "Opening local data folder…"}
        </p>
      )}
    </main>
  );
}

createRoot(document.getElementById("root")!).render(<App />);
