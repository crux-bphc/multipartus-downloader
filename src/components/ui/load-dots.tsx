import { useEffect, useState } from "react";

export const LoadingDots = () => {
    const [loadingDots, setLoadingDots] = useState("");
    
    useEffect(() => {
        const interval = setInterval(() => {
            setLoadingDots((prev) => (prev.length >= 3 ? "" : `${prev}.`));
        }, 300);

        return () => clearInterval(interval);
    }, []);

    return (<span className="absolute">{loadingDots}</span>)
}