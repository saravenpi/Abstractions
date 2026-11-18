(function () {
  const copyIcon =
    '<svg class="icon" xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true" focusable="false"><path d="M8 6h12v2H8zM4 2h12v2H4zm2 6h2v12H6zM2 4h2v12H2zm6 16h12v2H8zM20 8h2v12h-2zm-4-4h2v2h-2zM4 16h2v2H4z"/></svg>';
  const doneIcon =
    '<svg class="icon" xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true" focusable="false"><path d="M10 18H8v-2h2v2Zm-2-2H6v-2h2v2Zm4-2v2h-2v-2h2Zm-6 0H4v-2h2v2Zm8 0h-2v-2h2v2Zm2-2h-2v-2h2v2Zm2-2h-2V8h2v2Zm2-2h-2V6h2v2Z"/></svg>';

  function fallback(code) {
    const range = document.createRange();
    const selection = getSelection();
    range.selectNodeContents(code);
    selection.removeAllRanges();
    selection.addRange(range);
    const done = document.execCommand && document.execCommand("copy");
    return done ? Promise.resolve() : Promise.reject();
  }

  function write(code) {
    if (navigator.clipboard && window.isSecureContext) return navigator.clipboard.writeText(code.textContent);
    return fallback(code);
  }

  function show(button, text, icon) {
    button.innerHTML = icon + "<span>" + text + "</span>";
  }

  function attach(block) {
    const code = block.querySelector("code");
    const button = document.createElement("button");
    button.type = "button";
    button.className = "copy";
    button.setAttribute("aria-live", "polite");
    show(button, "Copy", copyIcon);
    button.addEventListener("click", function () {
      write(code)
        .then(() => show(button, "Copied", doneIcon))
        .catch(() => show(button, "Select and copy", copyIcon))
        .then(() => setTimeout(() => show(button, "Copy", copyIcon), 2000));
    });
    block.append(button);
  }

  document.querySelectorAll(".cmd").forEach(attach);
})();
