import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/api/shell";

(async () => {
  await listen("update_available", (event) => {
    const { status, current, latest } = event.payload;

    if (status === "update") {
      showUpdateModal(latest, current);
    } else if (status === "up_to_date") {
      console.log(`Up to date: ${current}`);
    } else {
      console.error("Error checking updates", event.payload);
    }
  });
})();

function showUpdateModal(latest, current) {
  const overlay = document.createElement("div");
  overlay.className =
    "fixed inset-0 flex items-center justify-center bg-black bg-opacity-50 z-50";

  overlay.innerHTML = `
    <div class="bg-white rounded-xl shadow-xl p-6 w-96 text-center">
      <h2 class="text-xl font-bold mb-4">Update available!</h2>
      <p>Latest: <b>${latest}</b><br/>Current: <b>${current}</b></p>
      <div class="mt-6 flex justify-center gap-4">
        <button id="openBtn" class="px-4 py-2 bg-blue-600 text-white rounded">Open GitHub Release</button>
        <button id="laterBtn" class="px-4 py-2 bg-gray-300 rounded">Later</button>
      </div>
    </div>
  `;

  document.body.appendChild(overlay);

  document.getElementById("openBtn").addEventListener("click", async () => {
    await open("https://github.com/eoliann/wup-web/releases/latest");
    overlay.remove();
  });

  document.getElementById("laterBtn").addEventListener("click", () =>
    overlay.remove()
  );
}
