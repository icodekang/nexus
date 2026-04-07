export default function Settings() {
  return (
    <div className="flex-1 overflow-y-auto p-6">
      <h1 className="text-xl font-semibold mb-6">Settings</h1>

      <div className="bg-white rounded-lg shadow p-6 space-y-6">
        <div>
          <h2 className="text-lg font-medium mb-4">API Configuration</h2>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                API Endpoint
              </label>
              <input
                type="text"
                defaultValue="https://api.novachat.com"
                className="w-full px-3 py-2 border rounded-lg"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                API Key
              </label>
              <input
                type="password"
                placeholder="sk-nova-..."
                className="w-full px-3 py-2 border rounded-lg"
              />
            </div>
          </div>
        </div>

        <div>
          <h2 className="text-lg font-medium mb-4">About</h2>
          <p className="text-gray-500">NovaChat v0.1.0</p>
        </div>
      </div>
    </div>
  )
}
