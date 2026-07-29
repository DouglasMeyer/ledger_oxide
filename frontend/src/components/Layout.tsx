import { NavLink, Outlet } from "react-router-dom";

const navItems = [
  { to: "/accounts", label: "Accounts" },
  { to: "/transactions", label: "Transactions" },
  { to: "/forecast", label: "Forecast" },
  { to: "/projections", label: "Projections" },
  { to: "/import", label: "Import" },
];

export default function Layout() {
  return (
    <div className="flex h-screen">
      <nav className="w-56 bg-gray-900 text-white flex flex-col shrink-0">
        <div className="px-4 py-4 text-lg font-bold border-b border-gray-700">
          Ledger Oxide
        </div>
        <ul className="flex-1 px-2 py-4 space-y-1">
          {navItems.map((item) => (
            <li key={item.to}>
              <NavLink
                to={item.to}
                className={({ isActive }) =>
                  `block px-3 py-2 rounded text-sm transition-colors ${
                    isActive
                      ? "bg-gray-700 text-white"
                      : "text-gray-300 hover:bg-gray-800 hover:text-white"
                  }`
                }
              >
                {item.label}
              </NavLink>
            </li>
          ))}
        </ul>
      </nav>
      <main className="flex-1 overflow-auto bg-gray-50 p-6">
        <Outlet />
      </main>
    </div>
  );
}
