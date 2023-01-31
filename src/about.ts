import { open } from "@tauri-apps/api/shell"

const license = document.getElementById("license");
const github = document.getElementById("github");

if (license) {
  license.onclick = () => {
    open("https://opensource.org/licenses/gpl-3.0.html");
  };
}

if (github) {
  github.onclick = () => {
    open("https://github.com/grantshandy/ipmap");
  };
}
