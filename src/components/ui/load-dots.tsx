import { useEffect, useState } from "react";

export const LoadingDots = ({ end }: { end: boolean }) => {
    const [loadingDots, setLoadingDots] = useState("");
    
    useEffect(() => {
        if (end) {
            setLoadingDots("");
            return;
        }

        const interval = setInterval(() => {
            setLoadingDots((prev) => (prev.length >= 3 ? "" : `${prev}.`));
        }, 300);

        return () => clearInterval(interval);
    }, [end]);

    return (<span className="absolute">{loadingDots}</span>)
}