package de.torsm.osrsnav

import com.runemate.game.api.hybrid.location.Coordinate
import com.runemate.game.api.hybrid.region.Players
import com.runemate.game.api.script.framework.LoopingBot

class TestBot : LoopingBot() {
    override fun onStart(vararg arguments: String?) {
        setLoopDelay(1000)
    }

    override fun onLoop() {
        val local = Players.getLocal()?.position ?: return
        val result = OsrsNav.buildBetween(local, Coordinate(3164, 3484, 0)) ?: return
        val test = convert(result)
        test.step()
    }
}