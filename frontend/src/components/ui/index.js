// Simple UI components for INTRASTAT module using Tailwind CSS

export const Card = ({ children, className = "" }) => (
  <div className={`bg-white shadow-sm rounded-lg border border-gray-200 ${className}`}>
    {children}
  </div>
);

export const CardHeader = ({ children, className = "" }) => (
  <div className={`px-6 py-4 border-b border-gray-200 ${className}`}>
    {children}
  </div>
);

export const CardContent = ({ children, className = "" }) => (
  <div className={`p-6 ${className}`}>
    {children}
  </div>
);

export const CardTitle = ({ children, className = "" }) => (
  <h3 className={`text-lg font-semibold text-gray-900 ${className}`}>
    {children}
  </h3>
);

export const Button = ({ 
  children, 
  variant = "default", 
  size = "md", 
  className = "", 
  ...props 
}) => {
  const variants = {
    default: "bg-blue-600 hover:bg-blue-700 text-white",
    outline: "border border-gray-300 hover:bg-gray-50 text-gray-700",
    destructive: "bg-red-600 hover:bg-red-700 text-white",
    secondary: "bg-gray-200 hover:bg-gray-300 text-gray-900"
  };
  
  const sizes = {
    sm: "px-3 py-1.5 text-sm",
    md: "px-4 py-2 text-sm",
    lg: "px-6 py-3 text-base"
  };

  return (
    <button
      className={`inline-flex items-center font-medium rounded-md transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 ${variants[variant]} ${sizes[size]} ${className}`}
      {...props}
    >
      {children}
    </button>
  );
};

export const Badge = ({ children, variant = "default", className = "" }) => {
  const variants = {
    default: "bg-gray-100 text-gray-800",
    secondary: "bg-gray-200 text-gray-900",
    success: "bg-green-100 text-green-800",
    destructive: "bg-red-100 text-red-800",
    outline: "border border-gray-300 text-gray-700"
  };

  return (
    <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${variants[variant]} ${className}`}>
      {children}
    </span>
  );
};

export const Input = ({ className = "", ...props }) => (
  <input
    className={`block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm ${className}`}
    {...props}
  />
);

export const Select = ({ children, className = "", ...props }) => (
  <select
    className={`block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm ${className}`}
    {...props}
  >
    {children}
  </select>
);

export const SelectTrigger = ({ children, className = "" }) => (
  <div className={`relative ${className}`}>
    {children}
  </div>
);

export const SelectContent = ({ children }) => children;
export const SelectItem = ({ children, ...props }) => (
  <option {...props}>{children}</option>
);
export const SelectValue = ({ placeholder }) => (
  <option value="" disabled>{placeholder}</option>
);

export const Table = ({ children, className = "" }) => (
  <div className="overflow-hidden shadow ring-1 ring-black ring-opacity-5 md:rounded-lg">
    <table className={`min-w-full divide-y divide-gray-300 ${className}`}>
      {children}
    </table>
  </div>
);

export const TableHeader = ({ children }) => (
  <thead className="bg-gray-50">
    {children}
  </thead>
);

export const TableBody = ({ children }) => (
  <tbody className="divide-y divide-gray-200 bg-white">
    {children}
  </tbody>
);

export const TableRow = ({ children, className = "" }) => (
  <tr className={className}>
    {children}
  </tr>
);

export const TableHead = ({ children, className = "" }) => (
  <th className={`px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider ${className}`}>
    {children}
  </th>
);

export const TableCell = ({ children, className = "" }) => (
  <td className={`px-6 py-4 whitespace-nowrap text-sm text-gray-900 ${className}`}>
    {children}
  </td>
);

export const Tabs = ({ children, value, onValueChange }) => (
  <div data-tabs-value={value} data-tabs-onchange={onValueChange}>
    {children}
  </div>
);

export const TabsList = ({ children, className = "" }) => (
  <div className={`flex space-x-1 rounded-lg bg-gray-100 p-1 ${className}`}>
    {children}
  </div>
);

export const TabsTrigger = ({ children, value, className = "" }) => (
  <button
    className={`inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1.5 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow ${className}`}
    data-value={value}
  >
    {children}
  </button>
);

export const TabsContent = ({ children, value, className = "" }) => (
  <div className={`mt-2 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 ${className}`} data-content={value}>
    {children}
  </div>
);

export const Alert = ({ children, className = "" }) => (
  <div className={`relative rounded-lg border p-4 ${className}`}>
    {children}
  </div>
);

export const AlertDescription = ({ children, className = "" }) => (
  <div className={`text-sm [&_p]:leading-relaxed ${className}`}>
    {children}
  </div>
);

export const Dialog = ({ children, open, onOpenChange }) => {
  if (!open) return null;
  
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="fixed inset-0 bg-black bg-opacity-50" onClick={() => onOpenChange(false)} />
      <div className="relative bg-white rounded-lg shadow-xl max-w-md w-full mx-4">
        {children}
      </div>
    </div>
  );
};

export const DialogContent = ({ children, className = "" }) => (
  <div className={`p-6 ${className}`}>
    {children}
  </div>
);

export const DialogHeader = ({ children, className = "" }) => (
  <div className={`flex flex-col space-y-1.5 pb-4 ${className}`}>
    {children}
  </div>
);

export const DialogTitle = ({ children, className = "" }) => (
  <h2 className={`text-lg font-semibold leading-none tracking-tight ${className}`}>
    {children}
  </h2>
);

export const DialogTrigger = ({ children, ...props }) => (
  <div {...props}>
    {children}
  </div>
);

export const Form = ({ children, onSubmit }) => (
  <form onSubmit={onSubmit}>
    {children}
  </form>
);

export const FormField = ({ children }) => children;
export const FormControl = ({ children }) => children;
export const FormItem = ({ children, className = "" }) => (
  <div className={`space-y-2 ${className}`}>
    {children}
  </div>
);
export const FormLabel = ({ children, className = "" }) => (
  <label className={`text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 ${className}`}>
    {children}
  </label>
);
export const FormMessage = ({ children, className = "" }) => (
  <p className={`text-sm font-medium text-red-500 ${className}`}>
    {children}
  </p>
);

export const Switch = ({ checked, onCheckedChange, className = "" }) => (
  <button
    type="button"
    className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 ${
      checked ? 'bg-blue-600' : 'bg-gray-200'
    } ${className}`}
    role="switch"
    aria-checked={checked}
    onClick={() => onCheckedChange(!checked)}
  >
    <span
      className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
        checked ? 'translate-x-6' : 'translate-x-1'
      }`}
    />
  </button>
);

export const Checkbox = ({ checked, onCheckedChange, className = "" }) => (
  <input
    type="checkbox"
    checked={checked}
    onChange={(e) => onCheckedChange(e.target.checked)}
    className={`h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded ${className}`}
  />
);

// Toast placeholder - in real app you'd use a toast library
export const toast = ({ title, description, variant = "default" }) => {
  console.log(`Toast: ${title} - ${description} (${variant})`);
  alert(`${title}: ${description}`);
};
