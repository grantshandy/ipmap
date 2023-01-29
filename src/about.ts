import { open } from "@tauri-apps/api/shell";

const btnGpl = document.getElementById("gpl");
const btnGit = document.getElementById("github");

if (btnGpl) {
  btnGpl.onclick = () => {
    open("https://opensource.org/licenses/gpl-3.0.html");
  };
}

if (btnGit) {
  btnGit.onclick = () => {
    open("https://github.com/grantshandy/ipmap");
  };
}
