function add_stream() {
  const name = prompt("Enter the display name of the scream", "");
  if (!name) return;

  const address = prompt("Enter the address of the stream", "");
  if (!address) return;

  document.getElementById("name").value = name;
  document.getElementById("address").value = address;
  document.getElementById("add_stream").submit();
}

function get_hash(string) {
  let hash = 0;
  for (let i = 0; i < string.length; i++) {
    hash += string.charCodeAt(i);
  }

  return hash;
}

document.addEventListener("DOMContentLoaded", () => {
  const colors = ["#f5e0dc", "#f2cdcd", "#f5c2e7", "#cba6f7", "#f38ba8", "#eba0ac", "#fab387", "#f9e2af", "#a6e3a1", "#94e2d5", "#89dceb", "#74c7ec", "#89b4fa", "#b4befe"];
  const buttons = document.querySelectorAll("#change_stream button");

  buttons.forEach((button) => {
    const hash = get_hash(button.textContent) % colors.length;
    button.style.setProperty("--stream-color", colors[hash]);
    button.textContent = button.textContent[0].toUpperCase();
  });
});
