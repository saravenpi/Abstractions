(function () {
  const buttons = Array.from(document.querySelectorAll("a.button[data-os]"));
  const prefix = "https://github.com/saravenpi/Abstractions/releases/download/";

  function detect() {
    const data = navigator.userAgentData;
    const text = ((data && data.platform) || navigator.platform || "") + " " + navigator.userAgent;
    if (/android|iphone|ipad|ipod/i.test(text)) return "";
    if (/win/i.test(text)) return "windows";
    if (/mac/i.test(text)) return "macos";
    if (/linux|x11/i.test(text)) return "linux";
    return "";
  }

  function highlight(os) {
    const button = buttons.find((item) => item.dataset.os === os);
    if (!button) return;
    const label = document.createElement("span");
    label.className = "yours";
    label.textContent = "Your system";
    button.classList.add("is-yours");
    (button.querySelector(".text") || button).append(label);
  }

  function megabytes(size) {
    return Math.round(size / 1048576) + " MB";
  }

  function link(button, assets) {
    const asset = assets.find((item) => item.name.endsWith(button.dataset.match));
    if (!asset || !String(asset.browser_download_url).startsWith(prefix)) return false;
    const meta = button.querySelector(".meta");
    button.href = asset.browser_download_url;
    meta.textContent = meta.textContent + ", " + megabytes(asset.size);
    return true;
  }

  function apply(release) {
    const date = new Date(release.published_at);
    const released = document.getElementById("released");
    const assets = Array.isArray(release.assets) ? release.assets : [];
    if (!/^v\d+\.\d+\.\d+$/.test(release.tag_name) || isNaN(date)) return;
    document.getElementById("version").textContent = release.tag_name;
    released.dateTime = date.toISOString().slice(0, 10);
    released.textContent = date.toLocaleDateString("en-GB", { day: "numeric", month: "long", year: "numeric" });
    if (buttons.map((button) => link(button, assets)).every(Boolean)) {
      document.getElementById("link-note").textContent = "Each button downloads the file from the latest release on GitHub.";
    }
  }

  highlight(detect());
  fetch("https://api.github.com/repos/saravenpi/Abstractions/releases/latest", {
    headers: { Accept: "application/vnd.github+json" },
  })
    .then((response) => (response.ok ? response.json() : Promise.reject(response.status)))
    .then(apply)
    .catch(function () {});
})();
